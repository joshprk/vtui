use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{
    context::{Context, EventContext},
    error::SendError,
    events::{ChannelRecv, Message},
    listeners::ListenerStore,
    transport::EventSink,
};

#[derive(Clone, Copy)]
pub struct ChannelSender<T: Send + 'static> {
    _marker: PhantomData<T>,
}

impl<T: Send + 'static> ChannelSender<T> {
    pub(crate) fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

pub struct ChannelReceiver<T: Send + 'static> {
    listeners: Rc<RefCell<ListenerStore>>,
    _marker: PhantomData<T>,
}

impl<T: Send + 'static> ChannelReceiver<T> {
    pub(crate) fn new(listeners: Rc<RefCell<ListenerStore>>) -> Self {
        Self { listeners, _marker: PhantomData }
    }

    pub fn listen(self, listener: impl FnMut(&mut EventContext<ChannelRecv<T>>) + 'static) {
        let listener = Box::new(listener);
        self.listeners.borrow_mut().push(listener)
    }
}

#[derive(Clone)]
pub struct ChannelSink<T: Send + 'static> {
    inner: EventSink,
    _marker: PhantomData<T>,
}

impl<T: Send + 'static> ChannelSink<T> {
    pub fn send(&self, value: T) -> Result<(), SendError> {
        let event = ChannelRecv { data: value };
        let msg = Message::new(event);
        self.inner.send(msg)
    }
}

pub(crate) fn create_blocking_channel<T: Send + 'static>(
    listeners: Rc<RefCell<ListenerStore>>,
) -> (ChannelSender<T>, ChannelReceiver<T>) {
    (ChannelSender::new(), ChannelReceiver::new(listeners))
}

pub(crate) fn spawn_blocking_task<T: Send + 'static>(
    ctx: &mut Context,
    sender: ChannelSender<T>,
    closure: impl FnOnce(ChannelSink<T>) + Send + 'static,
) {
    std::thread::spawn(move || {});
}
