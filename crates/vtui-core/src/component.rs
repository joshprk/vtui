use crate::{
    events::{Event, Message},
    listeners::{DrawContext, DrawListener, ListenerStore, UpdateContext},
};

pub struct Component {
    draw_listener: DrawListener,
    listeners: ListenerStore,
}

impl Component {
    pub fn draw(&mut self, listener: impl Fn(&DrawContext) + 'static) {
        self.draw_listener = Box::new(listener);
    }

    pub fn listen<E: Event>(&mut self, listener: impl FnMut(&UpdateContext<E>) + 'static) {
        self.listeners.push(Box::new(listener))
    }

    pub(crate) fn render(&self, ctx: &DrawContext) {
        (self.draw_listener)(ctx)
    }

    pub(crate) fn update(&mut self, msg: &Message) {
        self.listeners.dispatch(msg)
    }
}
