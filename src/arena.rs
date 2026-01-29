use ratatui::Frame;
use slotmap::{SlotMap, new_key_type};

use crate::{
    canvas::Canvas, component::Node, context::EventContext, layout::LogicalRect, transport::Event,
};

new_key_type! { struct NodeId; }

pub struct Arena {
    root: NodeId,
    nodes: SlotMap<NodeId, ArenaNode>,
    traversal: Vec<NodeId>,
}

impl From<Node> for Arena {
    fn from(node: Node) -> Self {
        let mut nodes = SlotMap::default();
        let root = nodes.insert(node.into());
        let traversal = compute_traversal(&nodes, root);

        Self {
            root,
            nodes,
            traversal,
        }
    }
}

impl Arena {
    pub fn render(&mut self, frame: &mut Frame) {
        let buf = frame.buffer_mut();

        for &id in self.traversal.iter() {
            let node = &mut self.nodes[id];
            let mut canvas = Canvas::new(node.rect, buf);

            node.node.render(&mut canvas);
        }
    }

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
    children: Vec<NodeId>,
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

fn compute_traversal(nodes: &SlotMap<NodeId, ArenaNode>, root: NodeId) -> Vec<NodeId> {
    let mut order = Vec::with_capacity(nodes.len());
    let mut stack = Vec::new();

    stack.push(root);

    while let Some(id) = stack.pop() {
        order.push(id);

        let node = &nodes[id];

        for &child in node.children.iter().rev() {
            stack.push(child);
        }
    }

    order
}
