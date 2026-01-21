use std::any::{Any, TypeId};

use crate::input::{KeyCode, MouseButton, MouseScrollDirection};

pub trait Event: Send + 'static {}

pub struct Message {
    event_type_id: TypeId,
    event: Box<dyn Any + Send>,
}

impl<E: Event> From<E> for Message {
    fn from(value: E) -> Self {
        Self {
            event_type_id: TypeId::of::<E>(),
            event: Box::new(value),
        }
    }
}

impl Message {
    pub fn new<E: Event>(event: E) -> Self {
        Self::from(event)
    }

    pub(crate) fn event_type_id(&self) -> TypeId {
        self.event_type_id
    }

    pub(crate) fn downcast_ref<E: Event>(&self) -> Option<&E> {
        self.event.downcast_ref::<E>()
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
