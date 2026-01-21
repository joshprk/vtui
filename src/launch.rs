use std::io;

use crate::{
    component::FactoryFn, drivers::Driver, error::RuntimeError, runtime::Runtime,
    transport::EventSource,
};

use crate::drivers::CrosstermDriver;

#[derive(Default)]
pub struct LaunchBuilder {}

impl LaunchBuilder {
    pub fn launch(self, factory: FactoryFn<()>) -> Result<(), RuntimeError> {
        let source = EventSource::new();
        let mut driver = CrosstermDriver::new(io::stdout());

        source.subscribe(&mut driver);
        driver.setup()?;

        let mut runtime = Runtime::new(factory, source);

        loop {
            runtime.draw(&mut driver)?;
            runtime.update();

            if runtime.should_exit() {
                break;
            }
        }

        driver.teardown()?;
        Ok(())
    }
}

pub fn launch(app: FactoryFn<()>) {
    LaunchBuilder::default().launch(app).unwrap()
}
