use std::io;

use thiserror::Error;
use vtui_core::{
    component::Component,
    driver::Driver,
    runtime::{EventSource, Runtime},
};

use crate::drivers::CrosstermDriver;

type FactoryFn = fn(&mut Component);

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Default)]
pub struct LaunchBuilder {}

impl LaunchBuilder {
    pub fn launch(self, factory: FactoryFn) -> Result<(), LaunchError> {
        let mut root = Component::default();
        factory(&mut root);

        let source = EventSource::new();
        let mut runtime = Runtime::new(root);
        let mut driver = CrosstermDriver::new(io::stdout());

        driver.setup()?;

        let terminal = driver.terminal();

        loop {
            terminal.draw(|f| {
                runtime.draw(f);
            })?;

            let event = source.recv();
            runtime.update(event);

            if runtime.should_exit() {
                break;
            }
        }

        driver.teardown()?;

        Ok(())
    }
}

pub fn launch(app: FactoryFn) -> Result<(), LaunchError> {
    LaunchBuilder::default().launch(app)
}
