use std::{
    io::{self, Write},
    sync::mpsc::Sender,
};

use crossterm::{
    event::{
        DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture, Event as CrosstermEvent, MouseButton as CrosstermMouseButton,
        MouseEvent as CrosstermMouseEvent, MouseEventKind,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use vtui_core::{
    driver::Driver,
    events::Message,
    input::{Input, MouseButton},
    runtime::EventProducer,
};

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

impl<W: Write> EventProducer for CrosstermDriver<W> {
    fn subscribe(tx: Sender<Message>) {
        loop {
            let crossterm_event = crossterm::event::read().unwrap();
            if let Some(input) = normalize_input(crossterm_event) {
                let message = input.to_message();
                let _ = tx.send(message);
            }
        }
    }
}

fn normalize_mouse_event(mouse_event: CrosstermMouseEvent) -> Option<Input> {
    fn normalize_button(button: CrosstermMouseButton) -> MouseButton {
        match button {
            CrosstermMouseButton::Left => MouseButton::Left,
            CrosstermMouseButton::Right => MouseButton::Right,
            CrosstermMouseButton::Middle => MouseButton::Middle,
        }
    }

    let x = mouse_event.column;
    let y = mouse_event.row;

    match mouse_event.kind {
        MouseEventKind::Down(button) => {
            let button = normalize_button(button);
            Some(Input::MouseDown { x, y, button })
        }
        MouseEventKind::Up(button) => {
            let button = normalize_button(button);
            Some(Input::MouseUp { x, y, button })
        }
        _ => None,
    }
}

fn normalize_input(event: CrosstermEvent) -> Option<Input> {
    match event {
        CrosstermEvent::Mouse(mouse_event) => normalize_mouse_event(mouse_event),
        _ => None,
    }
}
