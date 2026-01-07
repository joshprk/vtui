use crate::{
    context::{DrawContext, UpdateContext},
    events::{Event, Message},
    listeners::{DrawListener, ListenerStore},
};

#[derive(Default)]
pub struct Component {
    draw_listener: Option<DrawListener>,
    listeners: ListenerStore,
}

impl Component {
    pub fn draw(&mut self, listener: impl Fn(&mut DrawContext) + 'static) {
        self.draw_listener = Some(Box::new(listener));
    }

    pub fn listen<E: Event>(&mut self, listener: impl FnMut(&UpdateContext<E>) + 'static) {
        self.listeners.push(Box::new(listener))
    }

    pub(crate) fn render(&self, ctx: &mut DrawContext) {
        if let Some(ref draw_listener) = self.draw_listener {
            draw_listener(ctx)
        }
    }

    pub(crate) fn update(&mut self, msg: &Message) {
        self.listeners.dispatch(msg)
    }
}
