use std::ops::Deref;

use crate::events::Event;

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
}

#[derive(Default)]
pub(crate) struct Context {
    pub shutdown_requested: bool,
}
