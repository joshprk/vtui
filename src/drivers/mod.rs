use std::io;

use ratatui::{Terminal, prelude::Backend};

#[cfg(feature = "crossterm")]
pub use self::crossterm::CrosstermDriver;

#[cfg(feature = "crossterm")]
mod crossterm;

pub trait Driver {
    type Backend: Backend;

    fn setup(&mut self) -> io::Result<()>;
    fn teardown(self) -> io::Result<()>;
    fn terminal(&mut self) -> &mut Terminal<Self::Backend>;
}
