use alloc::rc::Rc;
use core::cell::RefCell;

use crate::{
    canvas::Canvas,
    context::EventContext,
    layout::{Flow, Inset, Measure, Placement},
    listeners::Listeners,
    state::{Scope, State, StateStore},
    transport::Event,
};

pub type BoxedRenderer = Box<dyn Fn(&mut Canvas)>;
pub type Factory<P = ()> = fn(Component, P) -> Node;

/// Types that are used when initializing a [`Node`].
pub trait Props: Clone + 'static {}

impl Props for () {}

/// An UI element with rendering and event listening behavior.
pub struct Component {
    node: RefCell<Node>,
}

impl Component {
    /// Determines whether this component should draw outside of its visual bounds.
    pub fn set_clipped(&self, clipped: bool) {
        self.node.borrow_mut().attributes.clipped = clipped;
    }

    /// Determines whether this component can capture focus.
    pub fn set_focusable(&self, focusable: bool) {
        self.node.borrow_mut().attributes.focusable = focusable;
    }

    /// Sets the [`Flow`] of this node.
    pub fn set_flow(&self, flow: Flow) {
        self.node.borrow_mut().attributes.flow = flow;
    }

    /// Sets the [`Placement`] of this node.
    pub fn set_placement(&self, placement: Placement) {
        self.node.borrow_mut().attributes.placement = placement;
    }

    /// Sets the canvas offset of this node.
    pub fn set_offset(&self, x: i32, y: i32) {
        self.node.borrow_mut().attributes.offset = (x, y);
    }

    /// Sets the default [`Measure`] of this node.
    pub fn set_measure(&self, measure: Measure) {
        self.node.borrow_mut().attributes.measure = measure;
    }

    /// Sets the margin of this node.
    pub fn set_margin(&self, margin: Inset) {
        self.node.borrow_mut().attributes.margin = margin;
    }

    /// Sets the padding of this node.
    pub fn set_padding(&self, padding: Inset) {
        self.node.borrow_mut().attributes.padding = padding;
    }

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

    /// Gets or creates a scoped context value.
    ///
    /// If a value of type `T` exists in an ancestor's scope, returns that handle.
    /// Otherwise, creates a new value using `T::default()` and provides it to descendants.
    pub fn context<T: Default + 'static>(&self) -> State<T> {
        if let Some(state) = self.node.borrow().scope.get::<T>() {
            state
        } else {
            self.set_context(T::default())
        }
    }

    /// Gets a scoped context value.
    ///
    /// Returns `None` if no ancestor has provided a value of type `T`.
    pub fn get_context<T: 'static>(&self) -> Option<State<T>> {
        self.node.borrow().scope.get::<T>()
    }

    /// Provides a scoped context value to descendants.
    ///
    /// The value is wrapped in [`State`] and made available to all descendant components.
    pub fn set_context<T: 'static>(&self, value: T) -> State<T> {
        let state = self.state(value);
        self.node.borrow_mut().scope.set::<T>(state);
        state
    }

    /// Builds the component into a [`Node`].
    pub fn compose<F>(self, builder: F) -> Node
    where
        F: Fn(&mut Ui) + 'static,
    {
        let mut node = self.node.into_inner();
        node.ui = Box::new(builder);
        node
    }

    pub(crate) fn new() -> Self {
        Self::with_scope(Rc::new(Scope::default()))
    }

    pub(crate) fn with_scope(parent_scope: Rc<Scope>) -> Self {
        let node = RefCell::new(Node::with_parent_scope(parent_scope));
        Self { node }
    }
}

/// A compiled description of an application's UI tree.
pub struct Node {
    attributes: NodeAttributes,
    state: StateStore,
    scope: Scope,
    draw_fn: Option<BoxedRenderer>,
    listeners: Listeners,
    ui: Box<dyn Fn(&mut Ui)>,
}

impl Node {
    fn new() -> Self {
        Self {
            attributes: NodeAttributes::default(),
            state: StateStore::default(),
            scope: Scope::default(),
            draw_fn: Option::default(),
            listeners: Listeners::default(),
            ui: Box::new(|_| {}),
        }
    }

    fn with_parent_scope(parent_scope: Rc<Scope>) -> Self {
        Self {
            scope: parent_scope.child(),
            ..Self::new()
        }
    }

    /// Returns this node's attributes.
    pub(crate) fn attributes(&self) -> &NodeAttributes {
        &self.attributes
    }

    pub(crate) fn attributes_mut(&mut self) -> &mut NodeAttributes {
        &mut self.attributes
    }

    pub(crate) fn compose(&self) -> Vec<Node> {
        let scope = Rc::new(self.scope.clone());
        let mut ui = Ui::with_scope(scope);
        (self.ui)(&mut ui);
        ui.children
    }

    /// Returns the flow of this node.
    pub(crate) fn flow(&self) -> Flow {
        self.attributes.flow
    }

    /// Returns the listeners of this node.
    pub(crate) fn listeners_mut(&mut self) -> &mut Listeners {
        &mut self.listeners
    }

    /// Returns the draw function of this node.
    pub(crate) fn renderer(&self) -> Option<&BoxedRenderer> {
        self.draw_fn.as_ref()
    }
}

/// Describes various properties for a [`Node`].
#[derive(Clone, Copy, Default)]
pub struct NodeAttributes {
    pub clipped: bool,
    pub focusable: bool,
    pub flow: Flow,
    pub placement: Placement,
    pub offset: (i32, i32),
    pub measure: Measure,
    pub margin: Inset,
    pub padding: Inset,
}

/// A builder for adding children to a component during composition.
///
/// Passed to the closure in [`Component::compose`] to construct the component's subtree.
pub struct Ui {
    children: Vec<Node>,
    scope: Rc<Scope>,
}

impl Ui {
    pub(crate) fn with_scope(scope: Rc<Scope>) -> Self {
        Self {
            children: Vec::new(),
            scope,
        }
    }

    /// Adds a new child to this node.
    pub fn child<P: Props>(&mut self, factory: Factory<P>, props: P) -> UiNode<'_> {
        let component = Component::with_scope(Rc::clone(&self.scope));
        self.children.push(factory(component, props));
        UiNode(self, self.children.len() - 1)
    }
}

/// A handle for configuring a child node's properties.
///
/// Returned by [`Ui::child`] to allow the parent to override certain attributes.
pub struct UiNode<'ui>(&'ui mut Ui, usize);

impl UiNode<'_> {
    /// Sets the [`Measure`] of this node.
    pub fn measure(&mut self, measure: Measure) -> &mut Self {
        self.node_mut().attributes_mut().measure = measure;
        self
    }

    fn node_mut(&mut self) -> &mut Node {
        self.0
            .children
            .get_mut(self.1)
            .expect("attempted to index an invalid UiNode")
    }
}
