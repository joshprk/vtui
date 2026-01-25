use std::ops::Deref;

use crate::{
    canvas::LogicalRect,
    events::{Event, MouseEvent},
};

pub struct EventContext<'e, E: Event> {
    event: &'e E,
    pass: UpdatePass<'e>,
}

impl<'e, E: Event> Deref for EventContext<'e, E> {
    type Target = E;

    fn deref(&self) -> &'e Self::Target {
        self.event
    }
}

impl<'e, E: Event> EventContext<'e, E> {
    pub(crate) fn new(event: &'e E, pass: UpdatePass<'e>) -> Self {
        Self { event, pass }
    }

    pub fn request_shutdown(&mut self) {
        self.pass.context.shutdown_requested = true;
    }
}

impl<'e, E: MouseEvent> EventContext<'e, E> {
    pub fn on_mouse_hit<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut EventContext<'e, E>),
    {
        let (x, y) = self.coords();
        let cursor = LogicalRect::new(x as i32, y as i32, 1, 1);

        if self.pass.rect.intersects(cursor) && !self.pass.state.mouse_hit_handled {
            self.pass.state.mouse_hit_handled = true;
            callback(self);
        }
    }
}

#[derive(Default)]
pub(crate) struct Context {
    pub shutdown_requested: bool,
}

pub(crate) struct UpdatePass<'e> {
    context: &'e mut Context,
    state: &'e mut UpdateState,
    rect: LogicalRect,
}

impl<'e> UpdatePass<'e> {
    pub fn new(context: &'e mut Context, state: &'e mut UpdateState, rect: LogicalRect) -> Self {
        Self {
            context,
            state,
            rect,
        }
    }
}

#[derive(Default)]
pub(crate) struct UpdateState {
    pub mouse_hit_handled: bool,
}
