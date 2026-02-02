use ratatui::Frame;
use slotmap::{SlotMap, new_key_type};

use crate::{
    canvas::Canvas,
    component::Node,
    context::EventContext,
    layout::{LogicalRect, Measure, compute_split},
    transport::Event,
};

new_key_type! { struct NodeId; }

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
    pub fn render(&mut self, frame: &mut Frame) {
        compute_layout(&mut self.nodes, self.root, frame.area().into());

        let buf = frame.buffer_mut();

        for &id in self.traversal.iter() {
            let node = &mut self.nodes[id];
            let mut canvas = Canvas::new(node.rect, buf);

            node.render(&mut canvas);
        }
    }

    /// Broadcasts an event to the node tree.
    pub fn update<E: Event>(&mut self, mut ctx: EventContext<E>) {
        for &id in self.traversal.iter().rev() {
            let node = &mut self.nodes[id];
            node.node.listeners_mut().dispatch(&mut ctx);
        }
    }
}

struct ArenaNode {
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
        let children = nodes[id].children.iter().collect::<Vec<_>>();
        let measures = children.iter().map(|(m, _)| *m);

        let splits = compute_split(flow, rect, measures);

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
