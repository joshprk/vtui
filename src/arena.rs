use ratatui::buffer::Buffer;
use slotmap::{SlotMap, new_key_type};

use crate::{
    canvas::{Canvas, LogicalRect},
    component::{Child, Node},
    context::Context,
    events::Message,
    layout::Measure,
};

new_key_type! { struct NodeId; }

#[derive(Default)]
pub(crate) struct Arena {
    root: NodeId,
    inner: SlotMap<NodeId, ArenaNode>,
}

impl Arena {
    pub fn new(root: Node) -> Self {
        let mut arena = Self::default();
        arena.root = arena.push(root);
        arena
    }

    pub fn update_for_each<F>(&mut self, mut update_fn: F)
    where
        F: FnMut(&mut ArenaNode),
    {
        let mut stack = vec![(self.root, false)];

        while let Some((id, visited)) = stack.pop() {
            if visited {
                let node = &mut self.inner[id];
                update_fn(node);
            } else {
                stack.push((id, true));
                for &(child, _) in self.inner[id].children.iter().rev() {
                    stack.push((child, false));
                }
            }
        }
    }

    pub fn draw_for_each<F>(&mut self, rect: LogicalRect, mut draw_fn: F)
    where
        F: FnMut(&ArenaNode),
    {
        let mut stack = vec![self.root];

        self.compute_layout(rect);

        while let Some(id) = stack.pop() {
            let node = &self.inner[id];
            draw_fn(node);

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

        id
    }
}

pub(crate) struct ArenaNode {
    inner: Node,
    parent: Option<NodeId>,
    children: Vec<(NodeId, Measure)>,
    rect: LogicalRect,
}

impl ArenaNode {
    pub(crate) fn render(&self, buffer: &mut Buffer) {
        if let Some(renderer) = &self.inner.get_renderer() {
            let mut canvas = Canvas::new(self.rect, buffer);
            renderer(&mut canvas);
        }
    }

    pub(crate) fn dispatch(&mut self, msg: &Message, ctx: &mut Context) {
        if let Some(listeners) = self.inner.get_listeners(msg) {
            listeners.dispatch(msg, ctx);
        }
    }
}
