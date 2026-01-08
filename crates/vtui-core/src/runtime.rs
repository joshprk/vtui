use std::sync::mpsc::{Receiver, Sender};

use ratatui::prelude::Backend;

use crate::{
    component::Component, canvas::Canvas, driver::Driver, error::RuntimeError, events::Message,
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

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
        let message = Message::new(Tick {});

        assert!(!*state.borrow());

        runtime.update(message);

        assert!(*state.borrow());
    }
}
