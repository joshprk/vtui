use std::time::{Duration, Instant};

use ratatui::prelude::Backend;

use crate::{
    arena::Arena,
    canvas::Canvas,
    component::{Component, FactoryFn},
    context::Context,
    driver::Driver,
    error::RuntimeError,
    events::Message,
    transport::EventSource,
};

pub struct Runtime {
    arena: Arena,
    context: Context,
    source: EventSource,
}

impl Runtime {
    pub fn new(factory: FactoryFn<()>, source: EventSource) -> Self {
        let context = Context::default();
        let root = Component::with_factory(factory, ());
        let arena = Arena::new(root);

        Self {
            arena,
            context,
            source,
        }
    }

    pub fn draw<D>(&self, driver: &mut D) -> Result<(), RuntimeError>
    where
        D: Driver,
        RuntimeError: From<<D::Backend as Backend>::Error>,
    {
        let terminal = driver.terminal();

        terminal.draw(|f| {
            let mut canvas = Canvas::new(f.area().into(), f.buffer_mut());

            for component in self.arena.iter_draw() {
                if let Some(draw_fn) = component.renderer() {
                    draw_fn(&mut canvas);
                }
            }
        })?;

        Ok(())
    }

    pub fn update(&mut self) {
        let deadline = Instant::now() + Duration::from_millis(16);
        let msg = self.source.recv();

        self.dispatch(&msg);

        while Instant::now() < deadline {
            let msg = self.source.recv_timeout(deadline - Instant::now());

            if let Some(msg) = msg {
                self.dispatch(&msg);
            }
        }
    }

    pub fn should_exit(&self) -> bool {
        self.context.shutdown_requested
    }
}

impl Runtime {
    fn dispatch(&mut self, message: &Message) {
        for component in self.arena.iter_update() {
            let listeners = component.listeners();

            if let Some(listeners) = listeners.get_mut(message) {
                listeners.dispatch(message, &mut self.context);
            }
        }
    }
}
