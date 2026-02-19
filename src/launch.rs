use core::time::Duration;
use std::{io, time::Instant};

use crate::{
    component::{Component, Factory},
    drivers::CrosstermDriver,
    errors::RuntimeError,
    runtime::Runtime,
    transport::MessageBus,
};

pub struct LaunchBuilder {
    update_budget: Duration,
}

impl Default for LaunchBuilder {
    fn default() -> Self {
        LaunchBuilder {
            update_budget: Duration::from_millis(16),
        }
    }
}

impl LaunchBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn launch(self, app: Factory) -> Result<(), RuntimeError> {
        let root = app(Component::default(), ());
        let bus = MessageBus::new();

        let mut runtime = Runtime::new(root);
        let mut driver = CrosstermDriver::new(io::stdout())?;

        loop {
            runtime.draw(&mut driver);

            let msg = bus.recv();

            let frame_start = Instant::now();
            let frame_budget = self.update_budget;

            runtime.update(msg);

            while let Some(msg) = {
                let remaining = frame_budget.saturating_sub(frame_start.elapsed());
                bus.recv_timeout(remaining)
            } {
                runtime.update(msg);
            }

            if runtime.should_exit() {
                break;
            }
        }

        Ok(())
    }
}

pub fn launch(app: Factory) {
    LaunchBuilder::default().launch(app).expect("runtime error")
}
