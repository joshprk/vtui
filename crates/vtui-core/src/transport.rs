use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use crate::{error::SendError, events::Message};

pub struct EventSource {
    tx: Sender<Message>,
    rx: Receiver<Message>,
}

impl Default for EventSource {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self { tx, rx }
    }
}

impl EventSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self, producer: &impl EventProducer) {
        let sink = EventSink(self.tx.clone());
        producer.spawn(sink);
    }

    pub(crate) fn recv(&self) -> Message {
        self.rx.recv().unwrap()
    }

    pub(crate) fn recv_timeout(&self, budget: Duration) -> Option<Message> {
        self.rx.recv_timeout(budget).ok()
    }
}

#[derive(Clone, Debug)]
pub struct EventSink(Sender<Message>);

impl EventSink {
    pub fn send(&self, value: Message) -> Result<(), SendError> {
        self.0.send(value).map_err(|_| SendError)
    }
}

pub trait EventProducer {
    fn subscribe(tx: EventSink);

    fn spawn(&self, tx: EventSink) {
        std::thread::spawn(move || {
            Self::subscribe(tx);
        });
    }
}
