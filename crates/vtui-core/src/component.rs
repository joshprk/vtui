use crate::{
    canvas::Canvas,
    context::{Context, EventContext},
    events::{Event, Message},
    listeners::{DrawListener, ListenerStore},
    state::{State, StateOwner},
};

pub type FactoryFn = fn(&mut Component) -> Inner;

#[derive(Default)]
pub struct Component {
    renderer: Option<DrawListener>,
    listeners: ListenerStore,
    state: StateOwner,
    inner: Inner,
}

impl Component {
    fn set_inner(&mut self, inner: Inner) {
        self.inner = inner;
    }
}

impl Component {
    pub(crate) fn with_factory(factory: FactoryFn) -> Self {
        let mut component = Component::default();
        let inner = factory(&mut component);
        component.inner = inner;
        component
    }

    pub fn draw(&mut self, listener: impl Fn(&mut Canvas) + 'static) {
        self.renderer = Some(Box::new(listener));
    }

    pub fn listen<E: Event>(&mut self, listener: impl FnMut(&mut EventContext<E>) + 'static) {
        self.listeners.push(Box::new(listener))
    }

    pub fn state<T: 'static>(&self, value: T) -> State<T> {
        self.state.insert(value)
    }
}

impl Component {
    pub(crate) fn render(&self, ctx: &mut Canvas) {
        if let Some(ref draw) = self.renderer {
            draw(ctx);
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
