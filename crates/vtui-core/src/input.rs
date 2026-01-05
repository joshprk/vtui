use crate::events::{Message, MouseDown, MouseDrag, MouseHover, MouseScroll, MouseUp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseScrollDirection {
    Up,
    Down,
    Left,
    Right,
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
    MouseHover {
        x: u16,
        y: u16,
    },
    MouseDrag {
        x: u16,
        y: u16,
        button: MouseButton,
    },
    Scroll {
        x: u16,
        y: u16,
        direction: MouseScrollDirection,
    },
}

impl Input {
    pub fn to_message(self) -> Message {
        match self {
            Input::MouseDown { x, y, button } => Message::new(MouseDown { x, y, button }),
            Input::MouseUp { x, y, button } => Message::new(MouseUp { x, y, button }),
            Input::MouseHover { x, y } => Message::new(MouseHover { x, y }),
            Input::MouseDrag { x, y, button } => Message::new(MouseDrag { x, y, button }),
            Input::Scroll { x, y, direction } => Message::new(MouseScroll { x, y, direction }),
        }
    }
}
