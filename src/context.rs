use core::ops::Deref;

use crate::transport::{Event, MouseEvent};

#[derive(Default)]
pub struct Context {
    pub shutdown_requested: bool,
}

pub struct EventContext<'d, E: Event> {
    event: &'d E,
    context: &'d mut Context,
    is_target: bool,
}

impl<E: Event> Deref for EventContext<'_, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.event
    }
}

impl<'d, E: Event> EventContext<'d, E> {
    pub fn new(event: &'d E, context: &'d mut Context, is_target: bool) -> Self {
        Self {
            event,
            context,
            is_target,
        }
    }
}

impl<E: Event> EventContext<'_, E> {
    /// Signals the runtime loop to shutdown.
    ///
    /// The runtime loop may defer or delay shutdown requests with discretion.
    pub fn request_shutdown(&mut self) {
        self.context.shutdown_requested = true;
    }
}

impl<E: MouseEvent> EventContext<'_, E> {
    /// Determines if the user clicked the component.
    ///
    /// A mouse hit is assigned to only one upper-most component containing the cursor.
    pub fn is_mouse_hit(&self) -> bool {
        self.is_target
    }
}
