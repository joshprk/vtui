use crate::events::{Message, MouseDown, MouseUp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub enum Input {
    MouseDown {
        x: u16,
        y: u16,
        button: MouseButton,
    },
    MouseUp {
        x: u16,
        y: u16,
        button: MouseButton,
    },
}

impl Input {
    pub fn to_message(self) -> Message {
        match self {
            Input::MouseDown { x, y, button } => Message::new(MouseDown { x, y, button }),
            Input::MouseUp { x, y, button } => Message::new(MouseUp { x, y, button }),
        }
    }
}
