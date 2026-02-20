use std::ops::Deref;

use crate::{
    arena::FrameData,
    context::{Command, Context},
    transport::Event,
};

pub struct EventHandler<'a, E: Event> {
    event: &'a E,
    context: &'a mut Context,
    data: FrameData,
}

impl<'a, E: Event> EventHandler<'a, E> {
    pub fn request_shutdown(&mut self) {
        self.context.enqueue(Command::Shutdown);
    }

    pub(crate) fn new(event: &'a E, context: &'a mut Context, data: FrameData) -> Self {
        Self {
            event,
            context,
            data,
        }
    }
}

impl<E: Event> Deref for EventHandler<'_, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.event
    }
}
