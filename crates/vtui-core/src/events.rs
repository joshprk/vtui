use std::any::{Any, TypeId};

use crate::input::MouseButton;

pub trait Event: Send + 'static {}

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
