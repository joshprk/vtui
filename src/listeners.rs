use core::any::{Any, TypeId};

use crate::{handler::EventHandler, transport::Event};

type EventListener<E> = Box<dyn FnMut(&mut EventHandler<E>)>;

#[derive(Default)]
pub struct Listeners {
    inner: Vec<Box<dyn Any>>,
    index: Vec<TypeId>,
}

impl Listeners {
    pub fn push<E, F>(&mut self, callback: F)
    where
        E: Event,
        F: FnMut(&mut EventHandler<E>) + 'static,
    {
        let id = TypeId::of::<E>();
        let cb = Box::new(callback) as EventListener<E>;

        if let Some(vec) = self.get_mut::<E>() {
            vec.push(cb);
        } else {
            let vec = vec![cb] as Vec<EventListener<E>>;
            self.index.push(id);
            self.inner.push(Box::new(vec));
        }
    }

    pub fn dispatch<E: Event>(&mut self, event: &mut EventHandler<E>) {
        if let Some(listeners) = self.get_mut::<E>() {
            listeners.iter_mut().for_each(|cb| cb(event));
        }
    }

    fn get_mut<E: Event>(&mut self) -> Option<&mut Vec<EventListener<E>>> {
        let idx = self.index.iter().position(|&id| id == TypeId::of::<E>())?;

        let slice = self.inner[idx]
            .downcast_mut::<Vec<EventListener<E>>>()
            .expect("listener indices malformed");

        Some(slice)
    }
}
