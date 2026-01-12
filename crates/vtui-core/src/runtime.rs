use std::time::{Duration, Instant};

use ratatui::prelude::Backend;

use crate::{
    canvas::Canvas, component::Component, context::Context, driver::Driver, error::RuntimeError,
    transport::EventSource,
};

pub struct Runtime {
    root: Component,
    context: Context,
}

impl Runtime {
    pub fn new(root: Component) -> Self {
        let context = Context::default();
        Self { root, context }
    }

    pub fn draw<D>(&self, driver: &mut D) -> Result<(), RuntimeError>
    where
        D: Driver,
        RuntimeError: From<<D::Backend as Backend>::Error>,
    {
        let terminal = driver.terminal();
        terminal.draw(|f| {
            let mut canvas = Canvas::new(f.area(), f.buffer_mut());
            self.root.render(&mut canvas);
        })?;
        Ok(())
    }

    pub fn update(&mut self, source: &EventSource) {
        let deadline = Instant::now() + Duration::from_millis(16);
        let msg = source.recv();

        self.root.update(&msg, &mut self.context);

        while Instant::now() < deadline {
            let msg = source.recv_timeout(deadline - Instant::now());
            if let Some(msg) = msg {
                self.root.update(&msg, &mut self.context);
            }
        }
    }

    pub fn should_exit(&self) -> bool {
        self.context.shutdown_requested
    }
}
