use std::{
    any::Any,
    io,
    sync::mpsc::{Receiver, Sender, channel},
    time::{Duration, Instant},
};

use ratatui::backend::Backend;
use thiserror::Error;

use crate::{drivers::Driver, element::Element};

pub trait Event: Any + Send {}

pub struct Runtime<T: Element> {
    bus: Receiver<Box<dyn Event>>,
    context: Context,
    root: T,
}

impl<T: Element> Runtime<T> {
    pub fn new(root: T) -> Self {
        let (context, bus) = Context::channel();
        Self { bus, context, root }
    }

    pub fn render<D>(&mut self, driver: &mut D) -> Result<(), RuntimeError>
    where
        D: Driver,
        RuntimeError: From<<<D as Driver>::Backend as Backend>::Error>,
    {
        driver.terminal().draw(|frame| {
            let rect = frame.area();
            let buffer = frame.buffer_mut();
            self.root.clone().render(&mut self.context, buffer, rect);
        })?;

        Ok(())
    }

    pub fn update(&mut self) {
        let deadline = Instant::now() + Duration::from_millis(16);
        let event = self.bus.recv().unwrap();
        self.root.clone().update(&mut self.context, &*event);

        while let Ok(event) = self.bus.recv_timeout(deadline - Instant::now()) {
            self.root.clone().update(&mut self.context, &*event);
        }
    }

    pub fn should_exit(&self) -> bool {
        self.context.shutdown_requested
    }

    pub fn handle(&self) -> &RuntimeHandle {
        &self.context.handle
    }
}

#[derive(Clone)]
pub struct RuntimeHandle {
    inner: Sender<Box<dyn Event>>,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub struct Context {
    pub handle: RuntimeHandle,
    pub shutdown_requested: bool,
}

impl Context {
    pub fn channel() -> (Self, Receiver<Box<dyn Event>>) {
        let (tx, rx) = channel();
        let context = Context {
            handle: RuntimeHandle { inner: tx },
            shutdown_requested: false,
        };
        (context, rx)
    }
}
