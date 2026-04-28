use std::ops::Deref;

use crate::{
    arena::FrameData,
    context::{Command, Context},
    layout::Region,
    transport::Event,
};

pub struct EventHandler<'a, E: Event> {
    event: &'a E,
    context: &'a mut Context,
    data: FrameData,
}

impl<E: Event> Deref for EventHandler<'_, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.event
    }
}

impl<'a, E: Event> EventHandler<'a, E> {
    pub fn rect(&self) -> Region {
        let abs = self.data.rect;
        Region::new(0, 0, abs.width, abs.height)
    }

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
