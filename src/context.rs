use core::ops::Deref;

use crate::{
    arena::NodeId,
    transport::{Event, MouseEvent},
};

#[derive(Default)]
pub struct Context {
    pub shutdown_requested: bool,
}

pub struct EventContext<'d, E: Event> {
    event: &'d E,
    context: &'d mut Context,
    current_node: NodeId,
}

impl<E: Event> Deref for EventContext<'_, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.event
    }
}

impl<'d, E: Event> EventContext<'d, E> {
    pub fn new(event: &'d E, context: &'d mut Context, current_node: NodeId) -> Self {
        Self {
            event,
            context,
            current_node,
        }
    }
}

impl<E: Event> EventContext<'_, E> {
    pub fn request_shutdown(&mut self) {
        self.context.shutdown_requested = true;
    }
}

impl<E: MouseEvent> EventContext<'_, E> {}
