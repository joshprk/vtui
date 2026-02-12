use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug)]
pub struct SendError;

impl<T> From<flume::SendError<T>> for SendError {
    fn from(_: flume::SendError<T>) -> Self {
        Self
    }
}
