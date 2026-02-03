use ratatui::prelude::Backend;

use crate::{
    arena::Arena,
    component::Node,
    context::Context,
    drivers::Driver,
    errors::RuntimeError,
    transport::{Dispatch, MessageBus},
};

pub struct Runtime {
    arena: Arena,
    context: Context,
    bus: MessageBus,
}

impl Runtime {
    pub fn new(node: Node, bus: MessageBus) -> Self {
        let arena = Arena::from(node);
        let handle = bus.handle();
        let context = Context::new(handle.clone());

        Self {
            arena,
            context,
            bus,
        }
    }

    pub fn draw<D>(&mut self, driver: &mut D) -> Result<(), RuntimeError>
    where
        D: Driver,
        RuntimeError: From<<<D as Driver>::Backend as Backend>::Error>,
    {
        let terminal = driver.terminal();
        terminal.draw(|f| self.arena.render(f))?;

        Ok(())
    }

    pub fn update(&mut self) {
        let msg = self.bus.recv();
        let dispatch = Dispatch::new(&mut self.arena, &mut self.context);
        msg.dispatch(dispatch);
    }

    pub fn should_exit(&self) -> bool {
        self.context.shutdown_requested()
    }
}
