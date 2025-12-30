use std::any::{Any, TypeId};

use ratatui::{Frame, buffer::Buffer, layout::Rect};

type DrawHandler = Box<dyn FnMut(DrawContext)>;

/// A marker trait for runtime signals.
///
/// An [`Event`] represents something that has already occurred, such as user input, a timer tick,
/// or the completion of an asynchronous task. Events are produced by the runtime and consumed
/// synchronously during the update phase.
///
/// Events carry no control flow and must not fail. All state transitions in response to an event
/// occur inside registered listeners during a runtime update.
pub trait Event {}

/// A runtime event where some producer wishes to continue the flow of time.
///
/// These events are emitted upon request by the runtime to drive time-based updates such as
/// animations, polling, or scheduled work.
///
/// The exact frequency and batching behavior are runtime-defined and may vary depending on
/// configuration.
pub struct Tick {}

impl Event for Tick {}

/// A builder which declares the properties of a component.
///
/// Components are consumed into a [`Runtime`] object which performs the declared behavior at
/// runtime.
#[derive(Default)]
pub struct Component {
    draw: Option<DrawHandler>
}

impl Component {
    /// Registers a listener for a specific [`Event`].
    pub fn listen<E: Event + 'static>(&mut self, mut listener: impl FnMut(&E) + 'static) {
        let type_id = TypeId::of::<E>();

        let wrapped = Box::new(move |event: &mut dyn Any| {
            if let Some(event) = event.downcast_mut::<E>() {
                listener(event);
            }
        });
    }

    /// Registers a draw handler that specifies how this component is rendered.
    pub fn draw(&mut self, listener: impl FnMut(DrawContext) + 'static) {
        self.draw = Some(Box::new(listener));
    }

    /// Builds the [`Component`] into a [`Runtime`], which can be used at runtime to perform the
    /// declared behavior of this [`Component`].
    pub fn build(self) -> Runtime {
        Runtime::new(self.draw)
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
/// The runtime is single-threaded and not [`Send`] or [`Sync`]. Concurrent systems,
/// such as async tasks or input streams, may enqueue events via channels, but
/// the runtime itself processes all events deterministically on one thread.
#[derive(Default)]
pub struct Runtime {
    draw: Option<DrawHandler>,
}

impl Runtime {
    pub fn new(draw: Option<DrawHandler>) -> Self {
        Self { draw }
    }

    pub fn update(&mut self) {
        // TODO
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let ctx = DrawContext {
            rect: frame.area(),
            buf: frame.buffer_mut(),
        };

        if let Some(draw) = &mut self.draw {
            draw(ctx);
        }
    }

    pub fn should_exit(&self) -> bool {
        false
    }
}
