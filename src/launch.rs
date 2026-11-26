use std::io;

use crate::{
    component::{Component, Factory},
    drivers::{CrosstermDriver, Driver},
    errors::RuntimeError,
    runtime::Runtime,
    transport::MessageBus,
};

/// Builder for configuring and launching an application.
#[derive(Default)]
pub struct LaunchBuilder {}

impl LaunchBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        LaunchBuilder::default()
    }

    /// Launches the application with the given root component.
    pub fn launch(self, app: Factory) -> Result<(), RuntimeError> {
        let node = app(Component::new(), ());

        let bus = MessageBus::new();
        let handle = bus.handle();
        let mut driver = CrosstermDriver::new(io::stdout())?;

        driver.setup()?;
        driver.spawn_event_handler(handle.clone());

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
/// Panics if the runtime encounters an error. Use [`LaunchBuilder`] for controlled error handling.
pub fn launch(app: Factory) {
    LaunchBuilder::new()
        .launch(app)
        .expect("app panicked unexpectedly");
}
