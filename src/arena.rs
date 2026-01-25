use ratatui::{Frame, buffer::Buffer};
use slotmap::{SlotMap, new_key_type};

use crate::{
    canvas::{Canvas, LogicalRect},
    component::{Child, Node},
    context::Context,
    events::Message,
    layout::Measure,
};

new_key_type! { struct NodeId; }

pub(crate) struct Arena {
    root: NodeId,
    inner: SlotMap<NodeId, ArenaNode>,
    traversal: Option<Vec<NodeId>>,
}

impl Arena {
    pub fn new(root: Node) -> Self {
        let mut arena = Self {
            root: NodeId::default(),
            inner: SlotMap::default(),
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

        for &id in order {
            let node = &mut self.inner[id];
            node.dispatch(msg, ctx);
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if self.traversal.is_none() {
            self.compute_traversal();
        }

        let rect = frame.area().into();
        let buf = frame.buffer_mut();

        self.compute_layout(rect);

        for &id in self.traversal.as_ref().unwrap() {
            let node = &self.inner[id];
            node.render(buf);
        }
    }
}

impl Arena {
    fn compute_layout(&mut self, rect: LogicalRect) {
        let root = self.root;
        self.inner[root].rect = rect;

        fn visit(arena: &mut Arena, id: NodeId) {
            let node = &arena.inner[id];

            let rect = node.rect;
            let composition = node.inner.composition();
            let children = node.children.clone();

            if children.is_empty() {
                return;
            }

            let rects = composition.split(rect, children.iter().map(|(_, m)| *m));

            debug_assert_eq!(rects.len(), children.len());

            for ((child_id, _), rect) in children.iter().zip(rects) {
                arena.inner[*child_id].rect = rect;
                visit(arena, *child_id);
            }
        }

        visit(self, root);
    }

    fn compute_traversal(&mut self) {
        let root = self.root;

        let mut order = Vec::new();
        let mut stack = vec![root];

        while let Some(id) = stack.pop() {
            order.push(id);

            let mut children = self.inner[id]
                .children
                .iter()
                .map(|(c, _)| *c)
                .collect::<Vec<_>>();

            children.sort_by_key(|&child_id| self.inner[child_id].inner.get_layer());

            for &child in children.iter().rev() {
                stack.push(child);
            }
        }

        self.traversal = Some(order);
    }

    fn push(&mut self, node: Node) -> NodeId {
        let id = self.inner.insert(ArenaNode {
            inner: node,
            parent: None,
            children: Vec::new(),
            rect: LogicalRect::new(0, 0, 0, 0),
        });

        let children = self.inner[id]
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
            self.inner[child_id].parent = Some(id);
            self.inner[id].children.push((child_id, measure));
        }

        self.traversal = None;

        id
    }
}

struct ArenaNode {
    inner: Node,
    parent: Option<NodeId>,
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

    fn dispatch(&mut self, msg: &Message, ctx: &mut Context) {
        if let Some(listeners) = self.inner.get_listeners(msg) {
            listeners.dispatch(msg, ctx);
        }
    }
}
