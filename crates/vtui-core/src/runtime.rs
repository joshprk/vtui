use std::sync::mpsc::{Receiver, Sender};

use ratatui::Frame;

use crate::{component::Component, events::Message, listeners::DrawContext};

pub struct Runtime {
    root: Component,
}

impl Runtime {
    pub fn new(root: Component) -> Self {
        Self { root }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let mut ctx = DrawContext {
            rect: frame.area(),
            buf: frame.buffer_mut(),
        };

        self.root.render(&mut ctx)
    }

    pub fn update(&mut self, msg: Message) {
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

    pub fn recv(&self) -> Option<Message> {
        self.rx.recv().ok()
    }
}
