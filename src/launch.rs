use std::io;

use crate::{
    component::{Factory, Node},
    drivers::{CrosstermDriver, Driver},
    errors::RuntimeError,
    runtime::Runtime,
    transport::MessageBus,
};

#[derive(Default)]
pub struct LaunchBuilder {}

impl LaunchBuilder {
    pub fn new() -> Self {
        LaunchBuilder::default()
    }

    pub fn launch(self, app: Factory) -> Result<(), RuntimeError> {
        let node = Node::from(app);

        let bus = MessageBus::new();
        let mut driver = CrosstermDriver::new(io::stdout())?;

        driver.setup()?;
        driver.spawn_event_handler(bus.sender());

        let mut runtime = Runtime::new(node, bus);

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

/// Launches an app with the given root component.
///
/// # Panics
///
/// This function is allowed to panic if the event loop errors. If you want to handle panics in a
/// controlled manner, use the [`LaunchBuilder`] manually.
pub fn launch(app: Factory) {
    LaunchBuilder::new()
        .launch(app)
        .expect("app panicked unexpectedly");
}
