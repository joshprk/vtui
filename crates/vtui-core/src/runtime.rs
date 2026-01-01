use std::{any::TypeId, collections::HashMap};

use ratatui::Frame;

use crate::{Component, DrawContext, DrawHandler, Listener, events::Event};

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
    root: Node,
}

impl Runtime {
    /// Creates a new [`Runtime`].
    pub fn new(root: Node) -> Self {
        Self { root }
    }

    /// Advances the state, observing [`Event`] as the most recent occurrence.
    pub fn update(&mut self, event: Box<dyn Event>) {
        let type_id = (*event).type_id();

        let Some(listeners) = self.root.inner.listeners.get_mut(&type_id) else {
            return;
        };

        for listener in listeners {
            // Dereference Box<dyn Event> to get &dyn Event for listener's expected signature
            listener(&*event, &Scope);
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

/// A handle identifying the current execution scope within the [`Runtime`].
pub struct Scope;

/// A compiled component item utilized by the runtime to define traversal.
#[derive(Default)]
pub struct Node {
    inner: Component,
}

impl Node {
    /// Draws the component and its children.
    fn draw(&mut self, ctx: DrawContext) {
        if let Some(draw) = &mut self.inner.draw {
            draw(ctx);
        }
    }
}

impl From<Component> for Node {
    fn from(value: Component) -> Self {
        Self { inner: value }
    }
}
