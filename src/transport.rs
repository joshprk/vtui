use std::{
    any::{Any, TypeId},
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
    time::Duration,
};

use crate::{error::SendError, events::Event};

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

pub struct EventSource {
    tx: Sender<Message>,
    rx: Receiver<Message>,
}

impl Default for EventSource {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self { tx, rx }
    }
}

impl EventSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self, producer: &mut impl EventProducer) {
        let sink = EventSink(self.tx.clone());
        producer.spawn(sink);
    }

    pub(crate) fn recv(&self) -> Message {
        self.rx.recv().unwrap()
    }

    pub(crate) fn recv_timeout(&self, budget: Duration) -> Option<Message> {
        self.rx.recv_timeout(budget).ok()
    }
}

#[derive(Clone, Debug)]
pub struct EventSink(Sender<Message>);

impl EventSink {
    pub fn send(&self, msg: Message) -> Result<(), SendError> {
        self.0.send(msg).map_err(|_| SendError)
    }
}

pub trait EventProducer {
    fn spawn(&mut self, tx: EventSink) -> JoinHandle<()>;
}
