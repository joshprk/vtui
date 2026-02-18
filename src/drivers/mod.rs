use ratatui::{Terminal, prelude::Backend};

pub trait Driver {
    type Backend: Backend;

    fn setup(&mut self) -> Result<(), <Self::Backend as Backend>::Error>;
    fn teardown(self) -> Result<(), <Self::Backend as Backend>::Error>;
    fn terminal(&mut self) -> &mut Terminal<Self::Backend>;
}
