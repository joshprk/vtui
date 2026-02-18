use core::{any::Any, time::Duration};

use crate::{arena::Arena, context::Context};

pub trait Event: Any + Send {}

pub struct Message {
    event: Box<dyn Event>,
    dispatch: fn(&mut Arena, Message, &mut Context),
}

impl Message {
    pub fn new<E: Event>(event: E) -> Self {
        fn trampoline<E: Event>(arena: &mut Arena, msg: Message, ctx: &mut Context) {
            let event = (msg.event as Box<dyn Any>)
                .downcast::<E>()
                .expect("failed to downcast event");
            arena.update(event.as_ref(), ctx);
        }

        let event = Box::new(event);
        let dispatch = trampoline::<E>;

        Self { event, dispatch }
    }

    pub fn dispatch(self, arena: &mut Arena, ctx: &mut Context) {
        let dispatch_fn = self.dispatch;
        dispatch_fn(arena, self, ctx);
    }
}

pub struct MessageBus {
    tx: flume::Sender<Message>,
    rx: flume::Receiver<Message>,
}

impl Default for MessageBus {
    fn default() -> Self {
        let (tx, rx) = flume::unbounded();
        Self { tx, rx }
    }
}

impl MessageBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sender(&self) -> &flume::Sender<Message> {
        &self.tx
    }

    pub fn recv(&self) -> Message {
        self.rx.recv().expect("channel closed unexpectedly")
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Option<Message> {
        self.rx.recv_timeout(timeout).ok()
    }
}
