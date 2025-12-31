use std::{
    any::TypeId,
    collections::HashMap, sync::{self, mpsc::{Receiver, Sender}},
};

use ratatui::{Frame, buffer::Buffer, layout::Rect};

use crate::events::Event;

pub mod events;

type DrawHandler = Box<dyn FnMut(DrawContext)>;
// TODO: listener dispatch performs downcast per listener invocation
// since listeners are already bucketed by TypeId, can be removed by storing Vec<Box<dyn FnMut(&E,
// &Scope>> behind a single erased wrapper
type Listener = Box<dyn FnMut(&dyn Event, &Scope)>;

/// A builder which declares the properties of a component.
///
/// Components are consumed into a [`Runtime`] object which performs the declared behavior at
/// runtime.
#[derive(Default)]
pub struct Component {
    draw: Option<DrawHandler>,
    listeners: HashMap<TypeId, Vec<Listener>>,
}

impl Component {
    /// Registers a listener for a specific [`Event`].
    pub fn listen<E: Event>(
        &mut self,
        mut listener: impl FnMut(UpdateContext<E>) + 'static,
    ) {
        let type_id = TypeId::of::<E>();
        let wrapped = Box::new(move |event: &dyn Event, scope: &Scope| {
            let event = event
                .as_any()
                .downcast_ref::<E>()
                .expect("TypeId mismatch");
            listener(UpdateContext { event, scope });
        });

        self.listeners.entry(type_id).or_default().push(wrapped);
    }

    /// Registers a draw handler that specifies how this component is rendered.
    pub fn draw(&mut self, listener: impl FnMut(DrawContext) + 'static) {
        self.draw = Some(Box::new(listener));
    }

    /// Builds the [`Component`] into a [`Runtime`], which can be used at runtime to perform the
    /// declared behavior of this [`Component`].
    pub fn build(self) -> Node {
        Node::from(self)
    }
}

/// A context container given to all component draw handlers.
///
/// This currently only contains the basic [`Rect`] and [`Buffer`] objects, but exists to support
/// forward compatibility for new features.
pub struct DrawContext<'a> {
    pub rect: Rect,
    pub buf: &'a mut Buffer,
}

/// A context container given to all component update listeners.
///
/// It provides access to the triggering [`Event`] and its associated [`Scope`].
pub struct UpdateContext<'a, E> {
    pub event: &'a E,
    pub scope: &'a Scope,
}

/// A handle identifying the current execution scope within the [`Runtime`].
pub struct Scope;

/// The execution engine for a `vtui` application.
///
/// A [`Runtime`] owns all state required to execute a component tree, including registered draw
/// handlers, event listeners, and internal queues. It is built from a fully-declared [`Component`]
/// and is responsible for driving the draw–update lifecycle.
///
/// # Event loop model
///
/// The runtime operates in a strict, single-threaded loop with well-defined phases:
///
/// Draws occur first in order to calculate layout for potential hit-testing events such as mouse
/// clicks. These occur synchronously from parent to children components.
///
/// A runtime update is performed immediately after, which blocks the event loop until it can
/// consume some event. This can range from user IO, promise completions/cancellations, tick events,
/// and more. It is also possible for the runtime to perform batching or coalescing of events in a
/// manner that is invariant to the draw function.
///
/// During a runtime update, a listener may potentially politely request shutdown. Once the runtime
/// is comfortable with a shutdown, the event loop exits.
///
/// # Concurrency
///
/// The runtime is single-threaded and not [`Send`] or [`Sync`]. Concurrent systems, such as asynchronous
/// tasks or input streams, may enqueue events via channels, but the runtime itself processes all events
/// deterministically on one thread.
#[derive(Default)]
pub struct Runtime {
    fps: Option<usize>,
    root: Node,
    inbox: EventSource,
}

impl Runtime {
    /// Creates a new [`Runtime`].
    pub fn new(root: Node, config: LaunchConfig) -> Self {
        let fps = config.fps;
        let inbox = EventSource::new();

        Self { fps, root, inbox }
    }

    /// Yields to the runtime so that it may consume incoming events.
    ///
    /// The runtime may choose to batch, drop, or coalesce events whose intermediate states are not
    /// semantically observable. For such events, only the most recent state within an update cycle
    /// is guaranteed to be delivered.
    pub fn update(&mut self) {
        let evt = self.inbox.recv();
        let type_id = (*evt).type_id();

        let Some(listeners) = self.root.listeners.get_mut(&type_id) else {
            return;
        };

        for listener in listeners {
            // Dereference Box<dyn Event> to get &dyn Event for listener's expected signature
            listener(&*evt, &Scope);
        }
    }

    /// Draws the runtime app on a mutable [`Frame`].
    ///
    /// This function reflects the current state produced by the most recent call to
    /// [`Runtime::update`].
    pub fn draw(&mut self, frame: &mut Frame) {
        self.root.draw(DrawContext {
            rect: frame.area(),
            buf: frame.buffer_mut(),
        });
    }

    /// Returns if an event loop should exit immediately.
    pub fn should_exit(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct EventSource {
    tx: Sender<Box<dyn Event>>,
    rx: Receiver<Box<dyn Event>>,
}

impl Default for EventSource {
    fn default() -> Self {
        let (tx, rx) = sync::mpsc::channel::<Box<dyn Event>>();

        Self { tx, rx }
    }
}

impl EventSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv(&mut self) -> Box<dyn Event> {
        self.rx.recv().unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct LaunchConfig {
    fps: Option<usize>,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self { fps: Some(60) }
    }
}

/// A compiled component item utilized by the runtime to define traversal.
#[derive(Default)]
pub struct Node {
    draw: Option<DrawHandler>,
    listeners: HashMap<TypeId, Vec<Listener>>,
}

impl Node {
    /// Draws the component and its children.
    fn draw(&mut self, ctx: DrawContext) {
        if let Some(draw) = &mut self.draw {
            draw(ctx);
        }
    }
}

impl From<Component> for Node {
    fn from(value: Component) -> Self {
        Self {
            draw: value.draw,
            listeners: value.listeners,
        }
    }
}
