use std::any::{Any, TypeId};

pub trait Event: AsAny {}

pub struct Tick {}
pub struct Tock {}

impl Event for Tick {}
impl Event for Tock {}

pub struct Message {
    pub type_id: TypeId,
    pub event: Box<dyn Event>,
}

pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Event> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
