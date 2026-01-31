use core::ops::Deref;

use crate::transport::Event;

#[derive(Default)]
pub struct Context {
    pub shutdown_requested: bool,
}

pub struct EventContext<'d, E: Event> {
    event: Box<E>,
    context: &'d mut Context,
}

impl<E: Event> Deref for EventContext<'_, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.event.as_ref()
    }
}

impl<'d, E: Event> EventContext<'d, E> {
    pub fn new(event: Box<E>, context: &'d mut Context) -> Self {
        Self { event, context }
    }
}

impl<E: Event> EventContext<'_, E> {
    pub fn request_shutdown(&mut self) {
        self.context.shutdown_requested = true;
    }
}
