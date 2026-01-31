use core::cell::RefCell;

use crate::{
    canvas::Canvas,
    context::EventContext,
    layout::Measure,
    listeners::Listeners,
    state::{State, StateStore},
    transport::Event,
};

pub type BoxedRenderer = Box<dyn Fn(&mut Canvas)>;
pub type Factory<P = ()> = fn(Component, P) -> Node;

/// Types that are used when initializing a [`Node`].
pub trait Props: Clone + 'static {}

impl Props for () {}

/// An UI element with rendering and event listening behavior.
#[derive(Default)]
pub struct Component {
    node: RefCell<Node>,
}

impl Component {
    /// Defines a render function for this component.
    pub fn draw(&self, renderer: impl Fn(&mut Canvas) + 'static) {
        let renderer = Box::new(renderer);
        self.node.borrow_mut().draw_fn = Some(renderer);
    }

    /// Adds a listener for an event.
    ///
    /// Listeners are sequential and single-threaded on the runtime thread. It is important to
    /// never run any blocking tasks to avoid UI jitters.
    pub fn listen<E>(&self, callback: impl FnMut(&mut EventContext<E>) + 'static)
    where
        E: Event,
    {
        self.node.borrow_mut().listeners.push(callback);
    }

    /// Initializes component state.
    ///
    /// State is owned by the component and can be passed down to children.
    pub fn state<T: 'static>(&self, value: T) -> State<T> {
        self.node.borrow_mut().state.insert(value)
    }
}

/// A compiled description of an application's UI tree.
#[derive(Default)]
pub struct Node {
    pub(crate) draw_fn: Option<BoxedRenderer>,
    pub(crate) listeners: Listeners,
    pub(crate) state: StateStore,
    pub(crate) children: Vec<(Measure, Node)>,
}

impl From<Component> for Node {
    fn from(component: Component) -> Self {
        component.node.into_inner()
    }
}

impl From<Factory> for Node {
    fn from(factory: Factory) -> Self {
        factory(Component::default(), ())
    }
}

impl Node {
    /// Creates a new node.
    pub fn new(component: Component) -> Self {
        Node::from(component)
    }

    /// Adds a static child to this node.
    pub fn child<P: Props>(mut self, measure: Measure, factory: Factory<P>, props: P) -> Self {
        let component = Component::default();
        let node = factory(component, props);
        self.children.push((measure, node));
        self
    }
}
