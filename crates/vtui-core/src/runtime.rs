use std::sync::mpsc::{Receiver, Sender};

use ratatui::prelude::Backend;

use crate::{
    canvas::Canvas, component::Component, driver::Driver, error::RuntimeError, events::Message,
};

pub struct Runtime {
    root: Component,
}

impl Runtime {
    pub fn new(root: Component) -> Self {
        Self { root }
    }

    pub fn draw<D>(&self, driver: &mut D) -> Result<(), RuntimeError>
    where
        D: Driver,
        RuntimeError: From<<D::Backend as Backend>::Error>,
    {
        let terminal = driver.terminal();
        terminal.draw(|f| {
            let mut canvas = Canvas::new(f.area(), f.buffer_mut());
            self.root.render(&mut canvas);
        })?;
        Ok(())
    }

    pub fn update(&mut self, source: &EventSource) {
        let msg = source.recv();
        self.root.update(&msg)
    }

    pub fn should_exit(&self) -> bool {
        false
    }
}

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

    pub fn recv(&self) -> Message {
        self.rx.recv().unwrap()
    }

    pub fn subscribe(&self, producer: &impl EventProducer) {
        producer.spawn(self.tx.clone());
    }
}

pub trait EventProducer {
    fn subscribe(tx: Sender<Message>);

    fn spawn(&self, tx: Sender<Message>) {
        std::thread::spawn(move || {
            Self::subscribe(tx);
        });
    }
}
