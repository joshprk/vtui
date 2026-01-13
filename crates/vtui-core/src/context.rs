use std::ops::Deref;

use crate::{
    channels::{ChannelSender, ChannelSink, spawn_blocking_task},
    events::Event,
};

pub struct EventContext<'rt, E: Event> {
    event: &'rt E,
    context: &'rt mut Context,
}

impl<'rt, E: Event> Deref for EventContext<'rt, E> {
    type Target = E;

    fn deref(&self) -> &'rt Self::Target {
        self.event
    }
}

impl<'rt, E: Event> EventContext<'rt, E> {
    pub(crate) fn new(event: &'rt E, context: &'rt mut Context) -> Self {
        Self { event, context }
    }

    pub fn request_shutdown(&mut self) {
        self.context.shutdown_requested = true;
    }

    pub fn spawn_blocking<T: Send + 'static>(
        &mut self,
        sender: ChannelSender<T>,
        closure: impl FnOnce(ChannelSink<T>) + Send + 'static,
    ) {
        spawn_blocking_task(self.context, sender, closure);
    }
}

#[derive(Default)]
pub(crate) struct Context {
    pub shutdown_requested: bool,
}
