use crate::events::{Message, MouseDown, MouseUp};

pub enum Input {
    MouseDown,
    MouseUp,
}

impl Input {
    pub fn to_message(self) -> Message {
        match self {
            Input::MouseDown => Message::new(MouseDown {}),
            Input::MouseUp => Message::new(MouseUp {}),
        }
    }
}
