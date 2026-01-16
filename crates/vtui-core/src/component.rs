use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use crate::{
    canvas::Canvas,
    context::{Context, EventContext},
    events::{Event, Message},
    listeners::{DrawListener, ListenerStore},
    state::{State, StateOwner},
};

pub type FactoryFn<P> = fn(Component, P) -> Node;

pub trait Props: Clone + 'static {}

impl Props for () {}

#[derive(Clone, Default)]
pub struct Component {
    inner: Rc<RefCell<Spec>>,
}

impl Component {
    fn get_inner_mut(&self) -> RefMut<'_, Spec> {
        self.inner.borrow_mut()
    }
}

impl Component {
    pub fn draw(&self, listener: impl Fn(&mut Canvas) + 'static) {
        self.get_inner_mut().renderer = Some(Box::new(listener));
    }

    pub fn listen<E: Event>(&self, listener: impl FnMut(&mut EventContext<E>) + 'static) {
        self.get_inner_mut().listeners.push(Box::new(listener));
    }

    pub fn state<T: 'static>(&self, value: T) -> State<T> {
        self.get_inner_mut().state.insert(value)
    }
}

#[derive(Default)]
pub(crate) struct Spec {
    pub renderer: Option<DrawListener>,
    pub listeners: ListenerStore,
    pub state: StateOwner,
}

pub struct Node {
    spec: Spec,
    children: Vec<Child>,
}

impl TryFrom<Component> for Node {
    type Error = Component;

    fn try_from(value: Component) -> Result<Self, Self::Error> {
        let Component { inner } = value;

        match Rc::try_unwrap(inner) {
            Ok(spec) => Ok(Self {
                spec: spec.into_inner(),
                children: Vec::default(),
            }),
            Err(inner) => Err(Component { inner }),
        }
    }
}

impl Node {
    pub fn from_component(value: Component) -> Result<Self, Component> {
        Self::try_from(value)
    }

    pub fn from_factory<P: Props>(factory: FactoryFn<P>, props: P) -> Self {
        factory(Component::default(), props)
    }

    pub fn add_static_child<P: Props>(&mut self, factory: FactoryFn<P>, props: P) {
        let factory = Box::new(move || Node::from_factory(factory, props.clone()));

        self.children.push(Child::Static(factory))
    }

    pub(crate) fn render(&self, mut canvas: Canvas) {
        if let Some(renderer) = &self.spec.renderer {
            renderer(&mut canvas);
        }
    }

    pub(crate) fn dispatch(&mut self, msg: &Message, ctx: &mut Context) {
        if let Some(listeners) = self.spec.listeners.get_mut(msg) {
            listeners.dispatch(msg, ctx);
        }
    }
}

pub enum Child {
    Static(Box<dyn Fn() -> Node>),
}
