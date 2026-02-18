use ratatui::prelude::Backend;

use crate::{
    arena::{Arena, Node},
    context::Context,
    drivers::Driver,
    errors::RuntimeError,
    transport::Message,
};

pub struct Runtime {
    arena: Arena,
    context: Context,
}

impl Runtime {
    pub fn new(node: Node) -> Self {
        Self {
            arena: Arena::from(node),
            context: Context::default(),
        }
    }

    pub fn draw<D>(&mut self, driver: &mut D) -> Result<(), RuntimeError>
    where
        D: Driver,
        RuntimeError: From<<<D as Driver>::Backend as Backend>::Error>,
    {
        let terminal = driver.terminal();

        terminal.draw(|f| {
            self.arena.render(f, &self.context);
        })?;

        Ok(())
    }

    pub fn update(&mut self, msg: Message) {
        msg.dispatch(&mut self.arena, &mut self.context);
    }
}
