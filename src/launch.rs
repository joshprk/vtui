use std::io;

use crate::{
    component::{Factory, Node},
    drivers::{CrosstermDriver, Driver},
    errors::RuntimeError,
    runtime::Runtime,
    transport::MessageBus,
};

pub fn launch(app: Factory) -> Result<(), RuntimeError> {
    let node = Node::from(app);
    let bus = MessageBus::new();

    let mut runtime = Runtime::new(node, bus);
    let mut driver = CrosstermDriver::new(io::stdout())?;

    driver.setup()?;

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
