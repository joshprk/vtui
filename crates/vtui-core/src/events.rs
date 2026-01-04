use std::any::{Any, TypeId};

pub trait Event: Send + 'static {}

pub struct Tick {}

impl Event for Tick {}

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
