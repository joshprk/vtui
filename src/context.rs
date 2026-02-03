use core::ops::Deref;

use crate::{
    arena::NodeId,
    transport::{Event, MessageSender, MouseEvent},
};

pub struct Context {
    handle: MessageSender,
    shutdown_requested: bool,
    target: Option<NodeId>,
    focused: Option<NodeId>,
}

impl Context {
    pub fn new(handle: MessageSender) -> Self {
        Self {
            handle,
            shutdown_requested: false,
            target: None,
            focused: None,
        }
    }

    pub fn handle(&self) -> &MessageSender {
        &self.handle
    }

    pub fn set_target(&mut self, target: Option<NodeId>) {
        self.target = target;
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
    current_node: NodeId,
}

impl<E: Event> Deref for EventContext<'_, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.event
    }
}

impl<'d, E: Event> EventContext<'d, E> {
    /// Creates a new event context.
    pub fn new(event: &'d E, context: &'d mut Context, current_node: NodeId) -> Self {
        Self {
            event,
            context,
            current_node,
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

    /// Determines if this component is focused.
    pub fn is_focused(&self) -> bool {
        self.context.focused == Some(self.current_node)
    }
}

impl<E: MouseEvent> EventContext<'_, E> {
    /// Determines if the user clicked this component.
    ///
    /// A mouse hit is assigned to only one upper-most component containing the cursor.
    pub fn is_mouse_hit(&self) -> bool {
        self.context.target == Some(self.current_node)
    }
}
