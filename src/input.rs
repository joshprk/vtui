use crate::{events::{
    KeyPress, KeyRelease, KeyRepeat, MouseDown, MouseDrag, MouseHover, MouseScroll,
    MouseUp, Resize,
}, transport::Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKeyCode {
    Shift,
    Ctrl,
    Alt,
    Super,
    Hyper,
    Meta,
    IsoLevel3Shift,
    IsoLevel5Shift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKeyDirection {
    Left,
    Right,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKeyCode {
    Play,
    Pause,
    PlayPause,
    Reverse,
    Stop,
    FastForward,
    Rewind,
    TrackNext,
    TrackPrevious,
    Record,
    LowerVolume,
    RaiseVolume,
    MuteVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Esc,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    KeypadBegin,
    Media(MediaKeyCode),
    Modifier(ModifierKeyCode, ModifierKeyDirection),
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
    MouseScroll {
        x: u16,
        y: u16,
        direction: MouseScrollDirection,
    },
    KeyPress {
        key: KeyCode,
    },
    KeyRepeat {
        key: KeyCode,
    },
    KeyRelease {
        key: KeyCode,
    },
    Resize {
        width: u16,
        height: u16,
    },
}

impl From<Input> for Message {
    fn from(value: Input) -> Self {
        match value {
            Input::MouseDown { x, y, button } => Message::new(MouseDown { x, y, button }),
            Input::MouseUp { x, y, button } => Message::new(MouseUp { x, y, button }),
            Input::MouseHover { x, y } => Message::new(MouseHover { x, y }),
            Input::MouseDrag { x, y, button } => Message::new(MouseDrag { x, y, button }),
            Input::MouseScroll { x, y, direction } => Message::new(MouseScroll { x, y, direction }),
            Input::KeyPress { key } => Message::new(KeyPress { key }),
            Input::KeyRepeat { key } => Message::new(KeyRepeat { key }),
            Input::KeyRelease { key } => Message::new(KeyRelease { key }),
            Input::Resize { width, height } => Message::new(Resize { width, height }),
        }
    }
}

impl Input {
    pub fn to_message(self) -> Message {
        Message::from(self)
    }
}
