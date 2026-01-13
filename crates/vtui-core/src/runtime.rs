use std::time::{Duration, Instant};

use ratatui::prelude::Backend;

use crate::{
    arena::Arena,
    canvas::Canvas,
    component::{Component, FactoryFn},
    context::Context,
    driver::Driver,
    error::RuntimeError,
    transport::EventSource,
};

pub struct Runtime {
    arena: Arena,
    context: Context,
    source: EventSource,
}

impl Runtime {
    pub fn new(factory: FactoryFn, source: EventSource) -> Self {
        let context = Context::default();
        let root = Component::with_factory(factory);
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
            let mut canvas = Canvas::new(f.area(), f.buffer_mut());

            for component in self.arena.iter_draw() {
                component.render(&mut canvas);
            }
        })?;

        Ok(())
    }

    pub fn update(&mut self) {
        let deadline = Instant::now() + Duration::from_millis(16);
        let msg = self.source.recv();

        self.arena.update(&msg, &mut self.context);

        while Instant::now() < deadline {
            let msg = self.source.recv_timeout(deadline - Instant::now());

            if let Some(msg) = msg {
                self.arena.update(&msg, &mut self.context);
            }
        }
    }

    pub fn should_exit(&self) -> bool {
        self.context.shutdown_requested
    }
}
