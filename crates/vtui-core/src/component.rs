use crate::{
    canvas::Canvas,
    context::{Context, EventContext},
    events::{Event, Message},
    listeners::{DrawListener, ListenerStore},
};

pub type FactoryFn = fn(&mut Component) -> Inner;

#[derive(Default)]
pub struct Component {
    draw_listener: Option<DrawListener>,
    listeners: ListenerStore,
    inner: Inner,
}

impl Component {
    fn set_inner(&mut self, inner: Inner) {
        self.inner = inner;
    }
}

impl Component {
    pub fn with_factory(factory: FactoryFn) -> Self {
        let mut component = Component::default();
        let inner = factory(&mut component);
        component.set_inner(inner);
        component
    }

    pub fn draw(&mut self, listener: impl Fn(&mut Canvas) + 'static) {
        self.draw_listener = Some(Box::new(listener));
    }

    pub fn listen<E: Event>(&mut self, listener: impl FnMut(&mut EventContext<E>) + 'static) {
        self.listeners.push(Box::new(listener))
    }

    pub(crate) fn render(&self, ctx: &mut Canvas) {
        if let Some(ref draw_listener) = self.draw_listener {
            draw_listener(ctx)
        }
    }

    pub(crate) fn update(&mut self, msg: &Message, ctx: &mut Context) {
        if let Some(listeners) = self.listeners.get_mut(msg) {
            listeners.dispatch(msg, ctx);
        }
    }
}

#[derive(Default)]
pub struct Inner {}
