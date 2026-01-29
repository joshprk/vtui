use std::io::Write;

use crossterm::{
    event::{
        DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    Terminal,
    prelude::{Backend, CrosstermBackend},
};

use crate::drivers::Driver;

pub struct CrosstermDriver<W: Write> {
    terminal: Terminal<CrosstermBackend<W>>,
}

impl<W: Write> Driver for CrosstermDriver<W> {
    type Backend = CrosstermBackend<W>;

    fn setup(&mut self) -> Result<(), <Self::Backend as Backend>::Error> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            EnableBracketedPaste,
            EnableFocusChange,
            EnableMouseCapture,
        )?;
        Ok(())
    }

    fn teardown(mut self) -> Result<(), <Self::Backend as Backend>::Error> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableBracketedPaste,
            DisableFocusChange,
            DisableMouseCapture,
        )?;
        Ok(())
    }

    fn terminal(&mut self) -> &mut ratatui::Terminal<Self::Backend> {
        &mut self.terminal
    }
}

impl<W: Write> CrosstermDriver<W> {
    pub fn new(writer: W) -> Result<Self, <<Self as Driver>::Backend as Backend>::Error> {
        let backend = CrosstermBackend::new(writer);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}
