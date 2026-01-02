use std::io;

use ratatui::{Terminal, prelude::Backend};

pub trait Driver {
    type Backend: Backend;

    fn setup(&mut self) -> io::Result<()>;
    fn teardown(self) -> io::Result<()>;
    fn terminal(&mut self) -> &mut Terminal<Self::Backend>;
}
