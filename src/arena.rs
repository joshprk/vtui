use ratatui::Frame;
use slotmap::{SlotMap, new_key_type};

use crate::{canvas::Canvas, component::Node, context::EventContext, transport::Event};

new_key_type! { struct NodeId; }

pub struct Arena {
    root: NodeId,
    nodes: SlotMap<NodeId, ArenaNode>,
}

impl From<Node> for Arena {
    fn from(value: Node) -> Self {
        let mut nodes = SlotMap::default();
        let root = nodes.insert(value.into());

        Self { root, nodes }
    }
}

impl Arena {
    pub fn render(&mut self, frame: &mut Frame) {
        let rect = frame.area().into();
        let buf = frame.buffer_mut();
        let mut canvas = Canvas::new(rect, buf);

        let root = &self.nodes[self.root];
        root.node.render(&mut canvas);
    }

    pub fn update<E: Event>(&mut self, mut ctx: EventContext<E>) {
        let root = &mut self.nodes[self.root];
        root.node.listeners_mut().dispatch(&mut ctx);
    }
}

struct ArenaNode {
    node: Node,
}

impl From<Node> for ArenaNode {
    fn from(node: Node) -> Self {
        Self { node }
    }
}
