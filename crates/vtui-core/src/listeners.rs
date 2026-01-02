use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use ratatui::{buffer::Buffer, layout::Rect};

use crate::events::{Event, Message};

pub(crate) type DrawListener = Box<dyn Fn(&mut DrawContext)>;
pub(crate) type Listener<E> = Box<dyn FnMut(&UpdateContext<E>)>;

pub(crate) trait ErasedListenerBucket {
    fn dispatch(&mut self, msg: &Message);
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Default)]
pub(crate) struct ListenerStore {
    inner: HashMap<TypeId, Box<dyn ErasedListenerBucket>>,
}

impl ListenerStore {
    pub fn dispatch(&mut self, msg: &Message) {
        let type_id = msg.type_id;

        if let Some(listeners) = self.inner.get_mut(&type_id) {
            listeners.dispatch(msg);
        }
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
    fn dispatch(&mut self, msg: &Message) {
        let event = msg.event.downcast_ref::<E>().expect("TypeId mismatch");

        let ctx = UpdateContext { event };

        for listener in &mut self.inner {
            listener(&ctx);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct DrawContext<'a> {
    pub buf: &'a mut Buffer,
    pub rect: Rect,
}

pub struct UpdateContext<'a, E> {
    pub event: &'a E,
}
