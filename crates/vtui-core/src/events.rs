use std::{any::Any, fmt::Debug};

/// A marker trait for runtime signals.
///
/// An [`Event`] represents something that has already occurred, such as user input, a timer tick,
/// or the completion of an asynchronous task. Events are produced by the runtime and consumed
/// synchronously during the update phase.
///
/// Events carry no control flow and must not fail. All state transitions in response to an event
/// occur inside registered listeners during a runtime update.
///
/// While we provide the ability to do so, creating your own events is not recommended and is not
/// considered idiomatic.
pub trait Event: AsAny + Debug + Send + Sync {}

/// A trait which implements a function `as_any` for all objects implementing [`Any`] and
/// [`Event`].
///
/// This is used for downcasting opaque events into their exact types during event dispatch.
pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Event> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A runtime event where some producer wishes to continue the flow of time.
///
/// These events are emitted upon request by the runtime to drive time-based updates such as
/// animations, polling, or scheduled work.
///
/// The exact frequency and batching behavior are runtime-defined and may vary depending on
/// configuration.
#[derive(Debug)]
pub struct Tick {}

impl Event for Tick {}
