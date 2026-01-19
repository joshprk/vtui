use std::cell::{RefCell, RefMut};

use crate::{
    canvas::{Canvas, LogicalRect},
    context::EventContext,
    events::{Event, Message},
    layout::{Axis, Measure, compute_split},
    listeners::{DrawListener, ErasedListenerBucket, ListenerStore},
    state::{State, StateOwner},
};

pub type FactoryFn<P> = fn(Component, P) -> Node;

pub trait Props: Clone + 'static {}

impl Props for () {}

#[derive(Default)]
pub struct Component {
    inner: RefCell<Spec>,
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

pub struct Node {
    spec: Spec,
    composition: Composition,
    pub layer: i32,
}

impl From<Component> for Node {
    fn from(value: Component) -> Self {
        let Component { inner } = value;

        Self {
            spec: inner.into_inner(),
            composition: Composition::default(),
            layer: 0,
        }
    }
}

impl Node {
    pub fn new(component: Component) -> Self {
        Self::from(component)
    }

    pub fn from_factory<P: Props>(factory: FactoryFn<P>, props: P) -> Self {
        factory(Component::default(), props)
    }

    pub(crate) fn get_renderer(&self) -> Option<&DrawListener> {
        self.spec.renderer.as_ref()
    }

    pub(crate) fn get_listeners(
        &mut self,
        msg: &Message,
    ) -> Option<&mut Box<dyn ErasedListenerBucket>> {
        self.spec.listeners.get_mut(msg)
    }

    pub(crate) fn composition(&self) -> &Composition {
        &self.composition
    }
}

impl Node {
    pub fn child<P: Props>(mut self, measure: Measure, factory: FactoryFn<P>, props: P) -> Self {
        let factory = Box::new(move || Node::from_factory(factory, props.clone()));
        self.composition.push(Child::Static(factory), measure);
        self
    }

    pub fn set_axis(mut self, axis: Axis) -> Self {
        self.composition.axis = axis;
        self
    }
}

#[derive(Default)]
pub(crate) struct Spec {
    pub renderer: Option<DrawListener>,
    pub listeners: ListenerStore,
    pub state: StateOwner,
}

pub(crate) enum Child {
    Static(Box<dyn Fn() -> Node>),
}

#[derive(Default)]
pub(crate) struct Composition {
    axis: Axis,
    children: Vec<(Child, Measure)>,
}

impl Composition {
    pub fn push(&mut self, child: Child, measure: Measure) {
        self.children.push((child, measure));
    }

    pub fn children(&self) -> impl Iterator<Item = &(Child, Measure)> {
        self.children.iter()
    }

    pub fn split(&self, area: LogicalRect, measures: &[Measure]) -> Vec<LogicalRect> {
        compute_split(self.axis, area, measures)
    }
}
