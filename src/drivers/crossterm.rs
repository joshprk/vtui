use std::io::{self, Write};

use crossterm::{
    event::{
        DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use vtui_core::driver::Driver;

pub struct CrosstermDriver<W: Write> {
    pub terminal: Terminal<CrosstermBackend<W>>,
}

impl<W: Write> CrosstermDriver<W> {
    pub fn new(writer: W) -> Self {
        let backend = CrosstermBackend::new(writer);
        let terminal = Terminal::new(backend).unwrap();
        Self { terminal }
    }
}

impl<W: Write> Driver for CrosstermDriver<W> {
    type Backend = CrosstermBackend<W>;

    fn setup(&mut self) -> io::Result<()> {
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

    fn teardown(mut self) -> io::Result<()> {
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

    fn terminal(&mut self) -> &mut Terminal<Self::Backend> {
        &mut self.terminal
    }
}
