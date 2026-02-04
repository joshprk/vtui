use core::ops::Deref;

use crate::{
    arena::NodeId,
    transport::{Event, MessageSender, MouseEvent},
};

pub enum Command {
    Shutdown,
    SetFocus(NodeId),
    ResignFocus(NodeId),
}

impl Command {
    pub fn reduce(self, ctx: &mut Context) {
        match self {
            Self::Shutdown => ctx.shutdown_requested = true,
            Self::SetFocus(id) => ctx.focused = Some(id),
            Self::ResignFocus(id) => {
                if ctx.focused == Some(id) {
                    ctx.focused = None;
                }
            }
        }
    }
}

pub struct Context {
    handle: MessageSender,
    shutdown_requested: bool,
    target: Option<NodeId>,
    focused: Option<NodeId>,
    command_buffer: Vec<Command>,
}

impl Context {
    pub fn new(handle: MessageSender) -> Self {
        Self {
            handle,
            shutdown_requested: false,
            target: None,
            focused: None,
            command_buffer: Vec::default(),
        }
    }

    pub fn commit(&mut self) {
        if !self.command_buffer.is_empty() {
            let commands = core::mem::take(&mut self.command_buffer);

            for cmd in commands {
                cmd.reduce(self);
            }
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

    pub fn enqueue(&mut self, cmd: Command) {
        self.command_buffer.push(cmd);
    }

    pub fn queued(&self) -> &[Command] {
        &self.command_buffer
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
    /// Requests focus for this component.
    ///
    /// Focus is assigned to the first component that requested it during an update.
    pub fn request_focus(&mut self) {
        for cmd in self.context.queued() {
            if matches!(cmd, Command::SetFocus(_)) {
                return;
            }
        }

        self.context.enqueue(Command::SetFocus(self.current_node));
    }

    /// Resigns focus for this component.
    ///
    /// Has no effect if this component is not currently focused. Resigning does not prevent another
    /// component from claiming focus in the same update.
    pub fn resign_focus(&mut self) {
        self.context
            .enqueue(Command::ResignFocus(self.current_node));
    }

    /// Signals the runtime loop to shutdown.
    ///
    /// The runtime loop may defer or delay shutdown requests with discretion.
    pub fn request_shutdown(&mut self) {
        self.context.enqueue(Command::Shutdown);
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
