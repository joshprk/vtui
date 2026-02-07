use ratatui::Frame;
use slotmap::{SlotMap, new_key_type};

use crate::{
    canvas::Canvas,
    component::{Node, NodeAttributes},
    context::{Context, EventContext},
    layout::{LogicalRect, Measure, compute_split},
    transport::Event,
};

/// Stores UI nodes in memory and dispatches requests to them.
pub struct Arena {
    root: NodeId,
    nodes: SlotMap<NodeId, ArenaNode>,
    traversal: Vec<NodeId>,
}

impl From<Node> for Arena {
    fn from(root: Node) -> Self {
        let mut nodes = SlotMap::default();
        let root = nodes.insert(root.into());

        remount_subtree(&mut nodes, root);

        let traversal = compute_traversal(&nodes, root);

        Self {
            root,
            nodes,
            traversal,
        }
    }
}

impl Arena {
    /// Draws the node tree on the given frame.
    pub fn render(&mut self, frame: &mut Frame, context: &Context) {
        compute_layout(&mut self.nodes, self.root, frame.area().into());

        let buf = frame.buffer_mut();

        for &id in self.traversal.iter() {
            let node = &self.nodes[id];
            let mut canvas = Canvas::new(node, buf, context, id);

            node.render(&mut canvas);
        }
    }

    /// Broadcasts an event to the node tree.
    pub fn update<E: Event>(&mut self, event: &E, context: &mut Context) {
        let target = event.target(self);
        context.set_target(target);

        for &id in self.traversal.iter().rev() {
            let node = &mut self.nodes[id];
            let mut ctx = EventContext::new(event, context, id);
            node.node.listeners_mut().dispatch(&mut ctx);
        }
    }

    /// Returns a reference to a node.
    pub fn get(&self, id: NodeId) -> Option<&ArenaNode> {
        self.nodes.get(id)
    }

    /// Returns an iterator in the forward traversal order.
    pub fn traverse(&self) -> impl DoubleEndedIterator<Item = (NodeId, &ArenaNode)> {
        self.traversal.iter().map(|&id| {
            let node = self.nodes.get(id).expect("traversal order has invalid id");
            (id, node)
        })
    }

    /// Sets the render offset of a node.
    ///
    /// # Panics
    ///
    /// Panics if the [`NodeId`] is invalid.
    pub fn set_offset(&mut self, id: NodeId, x: i32, y: i32) {
        let node = self
            .nodes
            .get_mut(id)
            .expect("set_offset received invalid id");
        node.node.set_offset(x, y);
    }
}

new_key_type! { pub struct NodeId; }

pub struct ArenaNode {
    node: Node,
    rect: LogicalRect,
    children: Vec<(Measure, NodeId)>,
}

impl From<Node> for ArenaNode {
    fn from(node: Node) -> Self {
        Self {
            node,
            rect: LogicalRect::zeroed(),
            children: Vec::new(),
        }
    }
}

impl ArenaNode {
    pub fn area(&self) -> LogicalRect {
        self.rect
    }

    pub fn attributes(&self) -> &NodeAttributes {
        self.node.attributes()
    }

    /// Renders the component into the frame buffer.
    fn render(&self, canvas: &mut Canvas) {
        if let Some(renderer) = &self.node.renderer() {
            renderer(canvas);
        }
    }
}

/// Assigns areas to nodes given their layout.
fn compute_layout(nodes: &mut SlotMap<NodeId, ArenaNode>, root: NodeId, viewport: LogicalRect) {
    let mut stack = vec![(root, viewport)];

    while let Some((id, rect)) = stack.pop() {
        nodes[id].rect = rect;

        let flow = nodes[id].node.flow();
        let placement = nodes[id].node.attributes().placement;
        let children = nodes[id].children.iter().collect::<Vec<_>>();
        let measures = children.iter().map(|(m, _)| *m);

        let splits = compute_split(flow, placement, rect, measures);

        for ((_, child_id), child_rect) in children.into_iter().zip(splits).rev() {
            stack.push((*child_id, child_rect));
        }
    }
}

/// Computes a pre-order DFS traversal order for a node tree.
fn compute_traversal(nodes: &SlotMap<NodeId, ArenaNode>, root: NodeId) -> Vec<NodeId> {
    let mut order = Vec::with_capacity(nodes.len());
    let mut stack = Vec::new();

    stack.push(root);

    while let Some(id) = stack.pop() {
        order.push(id);

        for &(_, child_id) in nodes[id].children.iter().rev() {
            stack.push(child_id);
        }
    }

    order
}

/// Destroys and recreates the children of the given [`Node`].
fn remount_subtree(nodes: &mut SlotMap<NodeId, ArenaNode>, root_id: NodeId) {
    remove_subtree(nodes, root_id);

    let children = nodes[root_id]
        .node
        .children()
        .iter()
        .map(|child_fn| child_fn())
        .collect::<Vec<_>>();

    for (measure, child) in children {
        let child_id = nodes.insert(child.into());
        nodes[root_id].children.push((measure, child_id));
        remount_subtree(nodes, child_id);
    }
}

/// Removes a node subtree recursively, leaving only its parent.
fn remove_subtree(nodes: &mut SlotMap<NodeId, ArenaNode>, root_id: NodeId) {
    let old = core::mem::take(&mut nodes[root_id].children);

    for (_, id) in old {
        remove_subtree(nodes, id);
        nodes.remove(id);
    }
}
