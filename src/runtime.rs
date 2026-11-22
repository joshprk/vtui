use std::{
    any::Any,
    io,
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};

use ratatui::backend::Backend;
use thiserror::Error;

use crate::{drivers::Driver, element::Element};

pub trait Event: Any {}

pub struct Runtime<T: Element> {
    bus: Receiver<Box<dyn Event>>,
    context: Context,
    root: T,
}

impl<T: Element> Runtime<T> {
    pub fn new(root: T) -> Self {
        let (handle, bus) = channel();
        let context = Context {
            handle,
            shutdown_requested: true,
        };
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
        while let Ok(event) = self.bus.recv_timeout(Duration::from_millis(16)) {
            self.root.clone().update(&mut self.context, &*event);
        }
    }

    pub fn should_exit(&self) -> bool {
        self.context.shutdown_requested
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub struct Context {
    handle: Sender<Box<dyn Event>>,
    pub shutdown_requested: bool,
}
