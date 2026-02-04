use std::time::{Duration, Instant};

use ratatui::prelude::Backend;

use crate::{
    arena::Arena,
    component::Node,
    context::Context,
    drivers::Driver,
    errors::RuntimeError,
    transport::{Dispatch, Message, MessageBus},
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
        let deadline = Instant::now() + Duration::from_millis(16);
        let msg = self.bus.recv();

        self.dispatch(msg);

        while let Some(msg) = self.bus.recv_timeout(deadline - Instant::now()) {
            self.dispatch(msg);
        }
    }

    pub fn should_exit(&self) -> bool {
        self.context.shutdown_requested()
    }

    fn dispatch(&mut self, msg: Message) {
        let dispatch = Dispatch::new(&mut self.arena, &mut self.context);
        msg.dispatch(dispatch);
        self.context.commit();
    }
}
