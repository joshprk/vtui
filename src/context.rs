use core::ops::Deref;

use crate::{
    arena::NodeId,
    events::{FocusChanged, Tick},
    transport::{Event, MessageSender, MouseEvent},
};

pub enum Command {
    Shutdown,
    SetFocus(NodeId),
    ResignFocus(NodeId),
    Tick,
}

impl Command {
    pub fn reduce(self, ctx: &mut Context) {
        match self {
            Self::Shutdown => ctx.shutdown_requested = true,
            Self::SetFocus(id) => {
                if ctx.focused != Some(id) {
                    ctx.focused = Some(id);
                    let _ = ctx.handle().send(FocusChanged {});
                }
            }
            Self::ResignFocus(id) => {
                if ctx.focused == Some(id) {
                    ctx.focused = None;
                    let _ = ctx.handle().send(FocusChanged {});
                }
            }
            Self::Tick => {
                let _ = ctx.handle().send(Tick {});
            }
        }
    }
}

pub struct Context {
    handle: MessageSender,
    target: Option<NodeId>,
    focused: Option<NodeId>,
    command_buffer: Vec<Command>,
    shutdown_requested: bool,
}

impl Context {
    pub fn new(handle: MessageSender) -> Self {
        Self {
            handle,
            target: None,
            focused: None,
            command_buffer: Vec::default(),
            shutdown_requested: false,
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

/// Provides event data and runtime actions within a listener callback.
///
/// `EventContext` cannot be constructed directly. It is given to listeners via
/// [`Component::listen`](crate::component::Component).
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
    pub(crate) fn new(event: &'d E, context: &'d mut Context, current_node: NodeId) -> Self {
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
    ///
    /// If successful, a [`FocusChanged`] event is emitted.
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
    ///
    /// If successful, a [`FocusChanged`] event is emitted.
    pub fn resign_focus(&mut self) {
        self.context
            .enqueue(Command::ResignFocus(self.current_node));
    }

    /// Requests a [`Tick`] event.
    ///
    /// This is useful for functionality requiring the flow of time, such as animations.
    pub fn request_tick(&mut self) {
        for cmd in self.context.queued() {
            if matches!(cmd, Command::Tick) {
                return;
            }
        }

        self.context.enqueue(Command::Tick);
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
