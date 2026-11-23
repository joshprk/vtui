use crate::{
    input::{KeyCode, MouseButton, MouseScrollDirection},
    runtime::Event,
};

/// A mouse button was pressed. Coordinates are terminal-relative.
pub struct MouseDown {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl Event for MouseDown {}

/// A mouse button was released. Coordinates are terminal-relative.
pub struct MouseUp {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl Event for MouseUp {}

/// The mouse cursor moved without a button held.
pub struct MouseHover {
    pub x: u16,
    pub y: u16,
}

impl Event for MouseHover {}

/// The mouse cursor moved with a button held.
pub struct MouseDrag {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl Event for MouseDrag {}

/// The mouse wheel was scrolled.
pub struct MouseScroll {
    pub x: u16,
    pub y: u16,
    pub direction: MouseScrollDirection,
}

impl Event for MouseScroll {}

/// A keyboard button was pressed.
pub struct KeyPress {
    pub key: KeyCode,
}

impl Event for KeyPress {}

/// A keyboard button was repeated.
///
/// Only emitted by terminals which report repeat events (such as via the Kitty keyboard protocol).
pub struct KeyRepeat {
    pub key: KeyCode,
}

impl Event for KeyRepeat {}

/// A keyboard button was released.
///
/// Only emitted by terminals which report release events (such as via the Kitty keyboard protocol).
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
