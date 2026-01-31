use crate::{
    input::{KeyCode, MouseButton, MouseScrollDirection},
    transport::{Event, MouseEvent},
};

pub struct Tick;

impl Event for Tick {}

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

pub struct MouseHover {
    pub x: u16,
    pub y: u16,
}

impl MouseEvent for MouseHover {
    fn coords(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

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

pub struct KeyPress {
    pub key: KeyCode,
}

impl Event for KeyPress {}

pub struct KeyRepeat {
    pub key: KeyCode,
}

impl Event for KeyRepeat {}

pub struct KeyRelease {
    pub key: KeyCode,
}

impl Event for KeyRelease {}

pub struct Resize {
    pub width: u16,
    pub height: u16,
}

impl Event for Resize {}
