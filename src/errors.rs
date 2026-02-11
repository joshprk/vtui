use std::io;

use thiserror::Error;

/// An error returned from [`LaunchBuilder::launch`](crate::launch::LaunchBuilder::launch).
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// An error returned from [`MessageSender::send`](crate::transport::MessageSender::send).
///
/// A send operation can only fail if the receiving end of a channel is disconnected.
#[derive(Debug)]
pub struct SendError;

impl<T> From<flume::SendError<T>> for SendError {
    fn from(_: flume::SendError<T>) -> Self {
        Self
    }
}

/// An error returned when attempting to access [`State`](crate::state::State) that is freed.
///
/// This is only possible through an self-referential anti-pattern `State<State<T>>`.
#[derive(Debug)]
pub struct StateError;

impl From<generational_box::BorrowError> for StateError {
    fn from(_: generational_box::BorrowError) -> Self {
        Self
    }
}

impl From<generational_box::BorrowMutError> for StateError {
    fn from(_: generational_box::BorrowMutError) -> Self {
        Self
    }
}
