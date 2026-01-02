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

#[cfg(test)]
mod tests {
    use std::{any::Any, cell::RefCell, rc::Rc};

    use crate::{
        component::Component,
        events::{Message, Tick},
        runtime::Runtime,
    };

    #[test]
    fn test_event_listen() {
        let mut root = Component::default();
        let state = Rc::new(RefCell::new(false));
        let state_c = state.clone();

        root.listen::<Tick>(move |_| {
            *state_c.borrow_mut() = true;
        });

        let mut runtime = Runtime::new(root);
        let event = Tick {};
        let message = Message {
            type_id: event.type_id(),
            event: Box::new(event),
        };

        assert!(!*state.borrow());

        runtime.update(message);

        assert!(*state.borrow());
    }
}
