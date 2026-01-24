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
            self.arena.render(f);
        })?;

        Ok(())
    }

    pub fn update(&mut self) {
        let deadline = Instant::now() + Duration::from_millis(16);
        let msg = self.source.recv();
        let ctx = &mut self.context;

        self.arena.dispatch(&msg, ctx);

        while Instant::now() < deadline {
            let msg = self.source.recv_timeout(deadline - Instant::now());

            if let Some(msg) = msg {
                self.arena.dispatch(&msg, ctx);
            }
        }
    }

    pub fn should_exit(&self) -> bool {
        self.context.shutdown_requested
    }
}
