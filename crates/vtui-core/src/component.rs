use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

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

pub struct Node {
    spec: Spec,
    composition: Composition,
    pub layer: i32,
}

impl TryFrom<Component> for Node {
    type Error = Component;

    fn try_from(value: Component) -> Result<Self, Self::Error> {
        let Component { inner } = value;

        match Rc::try_unwrap(inner) {
            Ok(spec) => Ok(Self {
                spec: spec.into_inner(),
                composition: Composition::default(),
                layer: 0,
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

    pub fn add_static_child<P: Props>(
        &mut self,
        factory: FactoryFn<P>,
        props: P,
        measure: Measure,
    ) {
        let factory = Box::new(move || Node::from_factory(factory, props.clone()));
        self.composition.push(Child::Static(factory), measure);
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
