use core::time::Duration;
use std::{io, time::Instant};

use crate::{
    component::{Component, Factory},
    drivers::{CrosstermDriver, Driver},
    errors::RuntimeError,
    runtime::Runtime,
    transport::MessageBus,
};

pub struct LaunchBuilder {
    frametime: Duration,
}

impl Default for LaunchBuilder {
    fn default() -> Self {
        LaunchBuilder {
            frametime: Duration::from_millis(16),
        }
    }
}

impl LaunchBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn frametime(mut self, budget: Duration) -> Self {
        self.frametime = budget;
        self
    }

    pub fn launch(self, app: Factory) -> Result<(), RuntimeError> {
        let root = app(Component::default(), ());
        let bus = MessageBus::new();
        let tx = bus.sender().clone();

        let mut runtime = Runtime::new(root);
        let mut driver = CrosstermDriver::new(io::stdout())?;

        driver.setup()?;
        driver.spawn_event_handler(tx);

        loop {
            runtime.draw(&mut driver)?;

            let msg = bus.recv();

            let frame_start = Instant::now();
            let frame_time = self.frametime;

            runtime.update(msg);

            while let Some(msg) = {
                let remaining = frame_time.saturating_sub(frame_start.elapsed());
                bus.recv_timeout(remaining)
            } {
                runtime.update(msg);
            }

            if runtime.should_exit() {
                break;
            }
        }

        driver.teardown()?;

        Ok(())
    }
}

pub fn launch(app: Factory) {
    LaunchBuilder::default().launch(app).expect("runtime error")
}
