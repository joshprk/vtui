use std::{any::{Any, TypeId}, collections::HashMap};

use crate::{UpdateContext, events::{Event, Message}};

type Listener<E: Event> = dyn FnMut(&UpdateContext<E>);
type ErasedListener = dyn FnMut(UpdateContext<dyn Event>);

#[derive(Default)]
pub struct ListenerStore {
    inner: HashMap<TypeId, Box<dyn Any>>
}

impl ListenerStore {
    pub fn push<E: Event>(&mut self, listener: Box<Listener<E>>) {
        let type_id = TypeId::of::<E>();
        self.inner
            .entry(type_id)
            .or_insert(Box::new(Listeners::<E>::new()))
            .downcast_mut::<Listeners<E>>()
            .expect("TypeId mismatch")
            .push(listener)
    }

    pub fn get<E: Event>(&self) -> Option<&Listeners<E>> {
        let type_id = TypeId::of::<E>();

        if let Some(erased) = self.inner.get(&type_id) {
            let listeners = erased
                .downcast_ref::<Listeners<E>>()
                .expect("TypeId mismatch");
            Some(listeners)
        } else {
            None
        }
    }

    pub fn dispatch(&mut self, message: &Message) -> Result<(), ()> {
        let type_id = message.type_id();

        if let Some(listeners) = self.inner.get_mut(&type_id) {
            listeners.downcast_ref::<Listeners<_>>();
        }

        Err(())
    }
}

struct Listeners<E: Event + Sized> {
    inner: Vec<Box<Listener<E>>>
}

impl<E: Event> Listeners<E> {
    fn new() -> Self {
        Self { inner: Vec::new() }
    }

    fn push(&mut self, value: Box<Listener<E>>) {
        self.inner.push(value)
    }

    pub fn dispatch(&mut self, ctx: UpdateContext<'_, E>) {
        for listener in &mut self.inner {
            listener(&ctx);
        }
    }
}
