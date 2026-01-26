use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{
    canvas::Canvas,
    context::{EventContext, UpdatePass},
    events::Event,
    transport::Message,
};

pub(crate) type DrawListener = Box<dyn Fn(&mut Canvas)>;
pub(crate) type Listener<E> = Box<dyn FnMut(&mut EventContext<E>)>;

pub(crate) trait ErasedListenerBucket {
    fn dispatch(&mut self, msg: &Message, pass: UpdatePass<'_>);
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Default)]
pub(crate) struct ListenerStore {
    inner: HashMap<TypeId, Box<dyn ErasedListenerBucket>>,
}

impl ListenerStore {
    pub fn get_mut(&mut self, msg: &Message) -> Option<&mut Box<dyn ErasedListenerBucket>> {
        let type_id = msg.event_type_id();
        self.inner.get_mut(&type_id)
    }

    pub fn push<E: Event>(&mut self, listener: Listener<E>) {
        let type_id = TypeId::of::<E>();
        self.inner
            .entry(type_id)
            .or_insert(Box::new(ListenerBucket::<E>::new()))
            .as_any_mut()
            .downcast_mut::<ListenerBucket<E>>()
            .expect("TypeId mismatch")
            .push(listener)
    }
}

#[derive(Default)]
pub(crate) struct ListenerBucket<E: Event> {
    inner: Vec<Listener<E>>,
}

impl<E: Event> ListenerBucket<E> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, listener: Listener<E>) {
        self.inner.push(listener)
    }
}

impl<E: Event> ErasedListenerBucket for ListenerBucket<E> {
    fn dispatch<'a>(&mut self, msg: &Message, pass: UpdatePass<'_>) {
        let event = msg.downcast_ref::<E>().expect("TypeId mismatch");
        let mut ctx = EventContext::new(event, pass);

        for listener in &mut self.inner {
            listener(&mut ctx);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
