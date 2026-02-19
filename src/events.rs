use crate::{
    input::{KeyCode, MouseButton, MouseScrollDirection},
    transport::{Event, MouseEvent},
};

/// An event requesting an additional update cycle before the next draw.
pub struct Tick;

impl Event for Tick {}

/// A mouse button was pressed.
pub struct MouseDown {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl MouseEvent for MouseDown {
    fn coords(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

/// A mouse button was released.
pub struct MouseUp {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl MouseEvent for MouseUp {
    fn coords(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

/// The mouse cursor moved without a button held.
pub struct MouseHover {
    pub x: u16,
    pub y: u16,
}

impl MouseEvent for MouseHover {
    fn coords(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

/// The mouse cursor moved with a button held.
pub struct MouseDrag {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl MouseEvent for MouseDrag {
    fn coords(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

/// The mouse wheel was scrolled.
pub struct MouseScroll {
    pub x: u16,
    pub y: u16,
    pub direction: MouseScrollDirection,
}

impl MouseEvent for MouseScroll {
    fn coords(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

/// A keyboard button was pressed.
pub struct KeyPress {
    pub key: KeyCode,
}

impl Event for KeyPress {}

/// A keyboard button was repeated.
///
/// # Compatibility
///
/// This event is only emitted on terminal emulators which support the Kitty keyboard protocol.
pub struct KeyRepeat {
    pub key: KeyCode,
}

impl Event for KeyRepeat {}

/// A keyboard button was released.
///
/// # Compatibility
///
/// This event is only emitted on terminal emulators which support the Kitty keyboard protocol.
pub struct KeyRelease {
    pub key: KeyCode,
}

impl Event for KeyRelease {}

/// The terminal emulator resized the buffer.
pub struct Resize {
    pub width: u16,
    pub height: u16,
}

impl Event for Resize {}

/// The focused component changed.
pub struct FocusChanged {}

impl Event for FocusChanged {}
