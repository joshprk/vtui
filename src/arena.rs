use ratatui::{Frame, buffer::Buffer};
use slotmap::{SlotMap, new_key_type};

use crate::{
    canvas::{Canvas, LogicalRect},
    component::{Child, Node},
    context::{Context, UpdatePass, UpdateState},
    layout::Measure,
    transport::Message,
};

new_key_type! { struct NodeId; }

pub(crate) struct Arena {
    root: NodeId,
    nodes: SlotMap<NodeId, ArenaNode>,
    traversal: Option<Vec<NodeId>>,
}

impl Arena {
    pub fn new(root: Node) -> Self {
        let mut arena = Self {
            root: NodeId::default(),
            nodes: SlotMap::default(),
            traversal: None,
        };

        arena.root = arena.push(root);
        arena.compute_traversal();

        arena
    }

    pub fn dispatch(&mut self, msg: &Message, ctx: &mut Context) {
        if self.traversal.is_none() {
            self.compute_traversal();
        }

        let order = self.traversal.as_ref().unwrap().iter().rev();
        let mut state = UpdateState::default();

        for &id in order {
            let node = &mut self.nodes[id];
            let pass = UpdatePass::new(ctx, &mut state, node.rect);
            node.dispatch(msg, pass);
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if self.traversal.is_none() {
            self.compute_traversal();
        }

        let rect = frame.area().into();
        let buf = frame.buffer_mut();

        self.compute_layout(rect);

        let order = self.traversal.as_ref().unwrap();

        for &id in order {
            let node = &self.nodes[id];
            node.render(buf);
        }
    }
}

impl Arena {
    fn compute_layout(&mut self, rect: LogicalRect) {
        let root = self.root;
        self.nodes[root].rect = rect;

        fn visit(arena: &mut Arena, id: NodeId) {
            let node = &arena.nodes[id];

            let rect = node.rect;
            let composition = node.inner.composition();
            let children = node.children.clone();

            if children.is_empty() {
                return;
            }

            let rects = composition.split(rect, children.iter().map(|(_, m)| *m));

            debug_assert_eq!(rects.len(), children.len());

            for ((child_id, _), rect) in children.iter().zip(rects) {
                arena.nodes[*child_id].rect = rect;
                visit(arena, *child_id);
            }
        }

        visit(self, root);
    }

    fn compute_traversal(&mut self) {
        let root = self.root;

        let mut order = Vec::with_capacity(self.nodes.len());
        let mut stack = vec![root];

        while let Some(id) = stack.pop() {
            order.push(id);

            let mut children = self.nodes[id]
                .children
                .iter()
                .map(|(c, _)| *c)
                .collect::<Vec<_>>();

            children.sort_by_key(|&child_id| self.nodes[child_id].inner.get_layer());

            for &child in children.iter().rev() {
                stack.push(child);
            }
        }

        self.traversal = Some(order);
    }

    fn push(&mut self, node: Node) -> NodeId {
        let id = self.nodes.insert(ArenaNode {
            inner: node,
            children: Vec::new(),
            rect: LogicalRect::new(0, 0, 0, 0),
        });

        let children = self.nodes[id]
            .inner
            .composition()
            .children()
            .map(|(child, measure)| {
                let child = match child {
                    Child::Static(factory) => factory(),
                };
                (child, *measure)
            })
            .collect::<Vec<(Node, Measure)>>();

        for (child, measure) in children {
            let child_id = self.push(child);
            let parent = &mut self.nodes[id];

            parent.children.push((child_id, measure));
        }

        self.traversal = None;

        id
    }
}

struct ArenaNode {
    inner: Node,
    children: Vec<(NodeId, Measure)>,
    rect: LogicalRect,
}

impl ArenaNode {
    fn render(&self, buffer: &mut Buffer) {
        if let Some(renderer) = &self.inner.get_renderer() {
            let mut canvas = Canvas::new(self.rect, buffer);
            renderer(&mut canvas);
        }
    }

    fn dispatch(&mut self, msg: &Message, pass: UpdatePass<'_>) {
        if let Some(listeners) = self.inner.get_listeners(msg) {
            listeners.dispatch(msg, pass);
        }
    }
}
