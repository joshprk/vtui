use std::any::{Any, TypeId};

use crate::input::{MouseButton, MouseScrollDirection};

pub trait Event: Send + 'static {}

pub struct Message {
    pub type_id: TypeId,
    pub event: Box<dyn Any + Send>,
}

impl Message {
    pub fn new(event: impl Event) -> Self {
        let type_id = event.type_id();
        let event = Box::new(event);
        Self { type_id, event }
    }
}

pub struct Tick {}

impl Event for Tick {}

pub struct MouseDown {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl Event for MouseDown {}

pub struct MouseUp {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl Event for MouseUp {}

pub struct MouseHover {
    pub x: u16,
    pub y: u16,
}

impl Event for MouseHover {}

pub struct MouseDrag {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl Event for MouseDrag {}

pub struct MouseScroll {
    pub x: u16,
    pub y: u16,
    pub direction: MouseScrollDirection,
}

impl Event for MouseScroll {}
