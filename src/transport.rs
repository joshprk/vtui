use core::any::Any;

use crate::{
    arena::{Arena, NodeId},
    context::Context,
    errors::SendError,
    layout::LogicalRect,
};

pub trait Event: Any + Send {
    fn target(&self, _arena: &Arena) -> Option<NodeId> {
        None
    }
}

pub trait MouseEvent: Event {
    fn coords(&self) -> (u16, u16);
}

impl<E: MouseEvent> Event for E {
    fn target(&self, arena: &Arena) -> Option<NodeId> {
        let (x, y) = self.coords();
        let cursor = LogicalRect::new(x as i32, y as i32, 1, 1);

        for (id, node) in arena.traverse().rev() {
            if node.area().intersects(cursor) {
                return Some(id);
            }
        }

        None
    }
}

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
        dispatch.arena.update(event.as_ref(), dispatch.context);
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
    tx: MessageSender,
    rx: flume::Receiver<Message>,
}

impl Default for MessageBus {
    fn default() -> Self {
        let (tx, rx) = flume::bounded(Self::DEFAULT_CAPACITY);
        let tx = MessageSender::from(tx);
        Self { tx, rx }
    }
}

impl MessageBus {
    const DEFAULT_CAPACITY: usize = 128;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle(&self) -> &MessageSender {
        &self.tx
    }

    pub fn recv(&self) -> Message {
        self.rx.recv().expect("bus closed unexpectedly")
    }
}

#[derive(Clone)]
pub struct MessageSender {
    tx: flume::Sender<Message>,
}

impl From<flume::Sender<Message>> for MessageSender {
    fn from(tx: flume::Sender<Message>) -> Self {
        Self { tx }
    }
}

impl MessageSender {
    pub fn send(&self, msg: impl Into<Message>) -> Result<(), SendError> {
        self.tx.send(msg.into()).map_err(|_| SendError)
    }
}
