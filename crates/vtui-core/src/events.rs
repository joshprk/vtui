use std::any::{Any, TypeId};

pub trait Event: 'static {}

pub struct Tick {}

impl Event for Tick {}

pub struct Message {
    pub type_id: TypeId,
    pub event: Box<dyn Any>,
}
