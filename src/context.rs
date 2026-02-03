use core::ops::Deref;

use crate::transport::{Event, MessageSender, MouseEvent};

pub struct Context {
    handle: MessageSender,
    shutdown_requested: bool,
}

impl Context {
    pub fn new(handle: MessageSender) -> Self {
        Self {
            handle,
            shutdown_requested: false,
        }
    }

    pub fn handle(&self) -> &MessageSender {
        &self.handle
    }

    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    pub fn request_shutdown(&mut self) {
        self.shutdown_requested = true;
    }
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
    /// Creates a new event context.
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
        self.context.request_shutdown()
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
