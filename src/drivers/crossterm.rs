use std::{
    io::{self, Write},
    thread::JoinHandle,
};

use crossterm::{
    event::{
        DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture, MouseEventKind,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use vtui_core::{
    driver::Driver,
    input::{
        Input, KeyCode, MediaKeyCode, ModifierKeyCode, ModifierKeyDirection, MouseButton,
        MouseScrollDirection,
    },
    transport::{EventProducer, EventSink},
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
    fn spawn(&mut self, tx: EventSink) -> JoinHandle<()> {
        std::thread::spawn(move || {
            loop {
                let event = crossterm::event::read().expect("crossterm::event::read failed");

                let msg = match normalize_input(event) {
                    Some(input) => input.to_message(),
                    None => continue,
                };

                if tx.send(msg).is_err() {
                    break;
                }
            }
        })
    }
}

fn normalize_input(event: crossterm::event::Event) -> Option<Input> {
    match event {
        crossterm::event::Event::Mouse(mouse_event) => normalize_mouse_event(mouse_event),
        crossterm::event::Event::Key(key_event) => normalize_key_event(key_event),
        crossterm::event::Event::Resize(width, height) => Some(Input::Resize { width, height }),
        _ => None,
    }
}

fn normalize_mouse_event(mouse_event: crossterm::event::MouseEvent) -> Option<Input> {
    fn normalize_button(button: crossterm::event::MouseButton) -> MouseButton {
        match button {
            crossterm::event::MouseButton::Left => MouseButton::Left,
            crossterm::event::MouseButton::Right => MouseButton::Right,
            crossterm::event::MouseButton::Middle => MouseButton::Middle,
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
        MouseEventKind::Moved => Some(Input::MouseHover { x, y }),
        MouseEventKind::Drag(button) => {
            let button = normalize_button(button);
            Some(Input::MouseDrag { x, y, button })
        }
        MouseEventKind::ScrollUp => Some(Input::MouseScroll {
            x,
            y,
            direction: MouseScrollDirection::Up,
        }),
        MouseEventKind::ScrollDown => Some(Input::MouseScroll {
            x,
            y,
            direction: MouseScrollDirection::Down,
        }),
        MouseEventKind::ScrollLeft => Some(Input::MouseScroll {
            x,
            y,
            direction: MouseScrollDirection::Left,
        }),
        MouseEventKind::ScrollRight => Some(Input::MouseScroll {
            x,
            y,
            direction: MouseScrollDirection::Right,
        }),
    }
}

fn normalize_key_event(key_event: crossterm::event::KeyEvent) -> Option<Input> {
    fn normalize_keycode(key_code: crossterm::event::KeyCode) -> Option<KeyCode> {
        match key_code {
            crossterm::event::KeyCode::Backspace => Some(KeyCode::Backspace),
            crossterm::event::KeyCode::Enter => Some(KeyCode::Enter),
            crossterm::event::KeyCode::Left => Some(KeyCode::Left),
            crossterm::event::KeyCode::Right => Some(KeyCode::Right),
            crossterm::event::KeyCode::Up => Some(KeyCode::Up),
            crossterm::event::KeyCode::Down => Some(KeyCode::Down),
            crossterm::event::KeyCode::Home => Some(KeyCode::Home),
            crossterm::event::KeyCode::End => Some(KeyCode::End),
            crossterm::event::KeyCode::PageUp => Some(KeyCode::PageUp),
            crossterm::event::KeyCode::PageDown => Some(KeyCode::PageDown),
            crossterm::event::KeyCode::Tab => Some(KeyCode::Tab),
            crossterm::event::KeyCode::BackTab => Some(KeyCode::Tab),
            crossterm::event::KeyCode::Delete => Some(KeyCode::Delete),
            crossterm::event::KeyCode::Insert => Some(KeyCode::Insert),
            crossterm::event::KeyCode::F(n) => Some(KeyCode::F(n)),
            crossterm::event::KeyCode::Char(c) => Some(KeyCode::Char(c)),
            crossterm::event::KeyCode::Esc => Some(KeyCode::Esc),
            crossterm::event::KeyCode::CapsLock => Some(KeyCode::CapsLock),
            crossterm::event::KeyCode::ScrollLock => Some(KeyCode::ScrollLock),
            crossterm::event::KeyCode::NumLock => Some(KeyCode::NumLock),
            crossterm::event::KeyCode::PrintScreen => Some(KeyCode::PrintScreen),
            crossterm::event::KeyCode::Pause => Some(KeyCode::Pause),
            crossterm::event::KeyCode::Menu => Some(KeyCode::Menu),
            crossterm::event::KeyCode::KeypadBegin => Some(KeyCode::KeypadBegin),
            crossterm::event::KeyCode::Media(key) => {
                let key = normalize_media_key(key);
                Some(KeyCode::Media(key))
            }
            crossterm::event::KeyCode::Modifier(key) => {
                let (key, direction) = normalize_modifier_key(key);
                Some(KeyCode::Modifier(key, direction))
            }
            crossterm::event::KeyCode::Null => None,
        }
    }

    let key = normalize_keycode(key_event.code)?;

    match key_event.kind {
        crossterm::event::KeyEventKind::Press => Some(Input::KeyPress { key }),
        crossterm::event::KeyEventKind::Repeat => Some(Input::KeyRepeat { key }),
        crossterm::event::KeyEventKind::Release => Some(Input::KeyRelease { key }),
    }
}

fn normalize_media_key(media_key_code: crossterm::event::MediaKeyCode) -> MediaKeyCode {
    match media_key_code {
        crossterm::event::MediaKeyCode::Play => MediaKeyCode::Play,
        crossterm::event::MediaKeyCode::Pause => MediaKeyCode::Pause,
        crossterm::event::MediaKeyCode::PlayPause => MediaKeyCode::PlayPause,
        crossterm::event::MediaKeyCode::Reverse => MediaKeyCode::Reverse,
        crossterm::event::MediaKeyCode::Stop => MediaKeyCode::Stop,
        crossterm::event::MediaKeyCode::FastForward => MediaKeyCode::FastForward,
        crossterm::event::MediaKeyCode::Rewind => MediaKeyCode::Rewind,
        crossterm::event::MediaKeyCode::TrackNext => MediaKeyCode::TrackNext,
        crossterm::event::MediaKeyCode::TrackPrevious => MediaKeyCode::TrackPrevious,
        crossterm::event::MediaKeyCode::Record => MediaKeyCode::Record,
        crossterm::event::MediaKeyCode::LowerVolume => MediaKeyCode::LowerVolume,
        crossterm::event::MediaKeyCode::RaiseVolume => MediaKeyCode::RaiseVolume,
        crossterm::event::MediaKeyCode::MuteVolume => MediaKeyCode::MuteVolume,
    }
}

fn normalize_modifier_key(
    modifier_key_code: crossterm::event::ModifierKeyCode,
) -> (ModifierKeyCode, ModifierKeyDirection) {
    match modifier_key_code {
        crossterm::event::ModifierKeyCode::LeftShift => {
            (ModifierKeyCode::Shift, ModifierKeyDirection::Left)
        }
        crossterm::event::ModifierKeyCode::LeftControl => {
            (ModifierKeyCode::Ctrl, ModifierKeyDirection::Left)
        }
        crossterm::event::ModifierKeyCode::LeftAlt => {
            (ModifierKeyCode::Alt, ModifierKeyDirection::Left)
        }
        crossterm::event::ModifierKeyCode::LeftSuper => {
            (ModifierKeyCode::Super, ModifierKeyDirection::Left)
        }
        crossterm::event::ModifierKeyCode::LeftHyper => {
            (ModifierKeyCode::Hyper, ModifierKeyDirection::Left)
        }
        crossterm::event::ModifierKeyCode::LeftMeta => {
            (ModifierKeyCode::Meta, ModifierKeyDirection::Left)
        }
        crossterm::event::ModifierKeyCode::RightShift => {
            (ModifierKeyCode::Shift, ModifierKeyDirection::Right)
        }
        crossterm::event::ModifierKeyCode::RightControl => {
            (ModifierKeyCode::Ctrl, ModifierKeyDirection::Right)
        }
        crossterm::event::ModifierKeyCode::RightAlt => {
            (ModifierKeyCode::Alt, ModifierKeyDirection::Right)
        }
        crossterm::event::ModifierKeyCode::RightSuper => {
            (ModifierKeyCode::Super, ModifierKeyDirection::Right)
        }
        crossterm::event::ModifierKeyCode::RightHyper => {
            (ModifierKeyCode::Hyper, ModifierKeyDirection::Right)
        }
        crossterm::event::ModifierKeyCode::RightMeta => {
            (ModifierKeyCode::Meta, ModifierKeyDirection::Right)
        }
        crossterm::event::ModifierKeyCode::IsoLevel3Shift => {
            (ModifierKeyCode::IsoLevel3Shift, ModifierKeyDirection::Left)
        }
        crossterm::event::ModifierKeyCode::IsoLevel5Shift => {
            (ModifierKeyCode::IsoLevel5Shift, ModifierKeyDirection::Left)
        }
    }
}
