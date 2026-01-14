use crate::{
    canvas::Canvas,
    context::EventContext,
    events::Event,
    listeners::{DrawListener, ListenerStore},
    state::{State, StateOwner},
};

pub type FactoryFn<P> = fn(&mut Component, P) -> Inner;

pub trait Props: 'static {}

impl Props for () {}

#[derive(Default)]
pub struct Component {
    renderer: Option<DrawListener>,
    listeners: ListenerStore,
    state: StateOwner,
    inner: Inner,
}

impl Component {
    pub(crate) fn with_factory<P: Props>(factory: FactoryFn<P>, props: P) -> Self {
        let mut component = Component::default();
        let inner = factory(&mut component, props);
        component.inner = inner;
        component
    }

    pub(crate) fn renderer(&self) -> Option<&DrawListener> {
        self.renderer.as_ref()
    }

    pub(crate) fn listeners(&mut self) -> &mut ListenerStore {
        &mut self.listeners
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

#[derive(Default)]
pub struct Inner {}
