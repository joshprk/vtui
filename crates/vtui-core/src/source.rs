use std::sync::mpsc::{Receiver, RecvError, Sender};

use crate::{events::Event, runtime::Runtime};

/// A synchronous event producer whose events can be fed to a [`Runtime`].
#[derive(Debug)]
pub struct EventSource {
    tx: Sender<Box<dyn Event>>,
    rx: Receiver<Box<dyn Event>>,
}

impl Default for EventSource {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<Box<dyn Event>>();
        Self { tx, rx }
    }
}

impl EventSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recv(&mut self) -> Result<Box<dyn Event>, RecvError> {
        self.rx.recv()
    }
}
