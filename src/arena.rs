use alloc::collections::VecDeque;

use ratatui::Frame;
use slotmap::{SlotMap, new_key_type};

use crate::{
    canvas::Canvas,
    component::{BoxedRenderer, Node},
    context::EventContext,
    layout::{LogicalRect, Measure},
    listeners::Listeners,
    state::StateStore,
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
        let root = populate_arena(&mut nodes, root);
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

            node.render(&mut canvas);
        }
    }

    pub fn update<E: Event>(&mut self, mut ctx: EventContext<E>) {
        for &id in self.traversal.iter().rev() {
            let node = &mut self.nodes[id];
            node.listeners.dispatch(&mut ctx);
        }
    }
}

struct ArenaNode {
    draw_fn: Option<BoxedRenderer>,
    listeners: Listeners,
    rect: LogicalRect,
    #[allow(dead_code)]
    state: StateStore,
    children: Vec<(Measure, NodeId)>,
}

impl ArenaNode {
    fn render(&self, canvas: &mut Canvas) {
        if let Some(renderer) = &self.draw_fn {
            renderer(canvas);
        }
    }
}

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

fn populate_arena(nodes: &mut SlotMap<NodeId, ArenaNode>, root: Node) -> NodeId {
    let mut queue = VecDeque::new();

    let Node {
        draw_fn,
        listeners,
        state,
        children,
    } = root;

    let root = nodes.insert(ArenaNode {
        draw_fn,
        listeners,
        rect: LogicalRect::zeroed(),
        state,
        children: Vec::new(),
    });

    queue.push_back((root, children));

    while let Some((parent_id, children)) = queue.pop_front() {
        for (measure, child) in children {
            let Node {
                draw_fn,
                listeners,
                state,
                children,
            } = child;

            let child_id = nodes.insert(ArenaNode {
                draw_fn,
                listeners,
                rect: LogicalRect::zeroed(),
                state,
                children: Vec::new(),
            });

            nodes[parent_id].children.push((measure, child_id));
            queue.push_back((child_id, children));
        }
    }

    root
}
