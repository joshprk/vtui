use std::time::{Duration, Instant};

use ratatui::prelude::Backend;

use crate::{
    arena::Arena,
    component::{FactoryFn, Node},
    context::Context,
    drivers::Driver,
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
        let root = Node::from_factory(factory, ());
        let arena = Arena::new(root);

        Self {
            arena,
            context,
            source,
        }
    }

    pub fn draw<D>(&mut self, driver: &mut D) -> Result<(), RuntimeError>
    where
        D: Driver,
        RuntimeError: From<<D::Backend as Backend>::Error>,
    {
        let terminal = driver.terminal();

        terminal.draw(|f| {
            let rect = f.area().into();
            let buffer = f.buffer_mut();

            self.arena.draw_for_each(rect, |node| {
                node.render(buffer);
            });
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
    fn dispatch(&mut self, msg: &Message) {
        self.arena.update_for_each(|node| {
            node.dispatch(msg, &mut self.context);
        })
    }
}
