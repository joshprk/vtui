use std::io;

use crate::{
    drivers::{CrosstermDriver, Driver},
    element::Element,
    runtime::{Runtime, RuntimeError},
};

struct LaunchBuilder;

impl LaunchBuilder {
    fn launch(self, root: impl Element) -> Result<(), RuntimeError> {
        let mut runtime = Runtime::new(root);
        let mut driver = CrosstermDriver::new(io::stdout())?;

        driver.setup()?;

        loop {
            runtime.render(&mut driver)?;
            runtime.update();

            if runtime.should_exit() {
                break;
            }
        }

        driver.teardown()?;
        Ok(())
    }
}

pub fn launch(root: impl Element) {
    LaunchBuilder.launch(root).unwrap();
}
