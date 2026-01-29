use core::any::Any;

use crate::{
    arena::Arena,
    context::{Context, EventContext},
    errors::SendError,
};

pub trait Event: Any {}

pub trait MouseEvent: Event {
    fn coords(&self) -> (u16, u16);
}

impl<E: MouseEvent> Event for E {}

pub struct Message {
    event: Box<dyn Event>,
    dispatch: fn(Message, Dispatch<'_>),
}

impl<E: Event> From<E> for Message {
    fn from(event: E) -> Self {
        Self {
            event: Box::new(event),
            dispatch: Self::dispatch_impl::<E>,
        }
    }
}

impl Message {
    pub fn new<E: Event>(event: E) -> Self {
        Self::from(event)
    }

    pub fn dispatch(self, dispatch: Dispatch<'_>) {
        (self.dispatch)(self, dispatch)
    }

    fn dispatch_impl<E: Event>(msg: Self, dispatch: Dispatch<'_>) {
        let event = (msg.event as Box<dyn Any>)
            .downcast::<E>()
            .expect("TypeId mismatch");
        let ctx = EventContext::new(event, dispatch.context);
        dispatch.arena.update(ctx);
    }
}

pub struct Dispatch<'d> {
    arena: &'d mut Arena,
    context: &'d mut Context,
}

impl<'d> Dispatch<'d> {
    pub fn new(arena: &'d mut Arena, context: &'d mut Context) -> Self {
        Self { arena, context }
    }
}

pub struct MessageBus {
    tx: flume::Sender<Message>,
    rx: flume::Receiver<Message>,
}

impl Default for MessageBus {
    fn default() -> Self {
        let (tx, rx) = flume::bounded(Self::DEFAULT_CAPACITY);
        Self { tx, rx }
    }
}

impl MessageBus {
    const DEFAULT_CAPACITY: usize = 128;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn sender(&self) -> MessageSender {
        MessageSender::from(self)
    }

    pub fn recv(&self) -> Message {
        self.rx.recv().expect("bus closed unexpectedly")
    }
}

pub struct MessageSender {
    tx: flume::Sender<Message>,
}

impl From<&MessageBus> for MessageSender {
    fn from(value: &MessageBus) -> Self {
        Self {
            tx: value.tx.clone(),
        }
    }
}

impl MessageSender {
    pub fn send(&self, msg: impl Into<Message>) -> Result<(), SendError> {
        self.tx.send(msg.into()).map_err(|_| SendError)
    }
}
