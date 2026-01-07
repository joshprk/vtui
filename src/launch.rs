use std::io;

use vtui_core::{
    component::{Component, FactoryFn},
    driver::Driver,
    runtime::{EventSource, Runtime},
};

use crate::drivers::CrosstermDriver;

#[derive(Default)]
pub struct LaunchBuilder {}

impl LaunchBuilder {
    pub fn launch(self, factory: FactoryFn) {
        let root = Component::with_factory(factory);
        let source = EventSource::new();
        let mut runtime = Runtime::new(root);
        let mut driver = CrosstermDriver::new(io::stdout());

        source.subscribe(&driver);
        driver.setup();

        loop {
            runtime.draw(&mut driver);
            runtime.update(source.recv());

            if runtime.should_exit() {
                break;
            }
        }

        driver.teardown();
    }
}

pub fn launch(app: FactoryFn) {
    LaunchBuilder::default().launch(app)
}
