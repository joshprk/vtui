use ratatui::buffer::Buffer;
use slotmap::{SlotMap, new_key_type};

use crate::{
    canvas::{Canvas, LogicalRect},
    component::{Child, Node},
    context::Context,
    events::Message,
};

new_key_type! { struct NodeId; }

#[derive(Default)]
pub(crate) struct Arena {
    roots: Vec<NodeId>,
    inner: SlotMap<NodeId, ArenaNode>,
}

impl Arena {
    pub fn new(root: Node) -> Self {
        let mut arena = Self::default();
        arena.push(root, None);
        arena
    }

    pub fn update_for_each<F>(&mut self, mut update_fn: F)
    where
        F: FnMut(&mut ArenaNode),
    {
        let mut stack = Vec::new();

        for &root in self.roots.iter().rev() {
            stack.push((root, false));
        }

        while let Some((id, visited)) = stack.pop() {
            if visited {
                let node = &mut self.inner[id];
                update_fn(node);
            } else {
                stack.push((id, true));
                for &child in self.inner[id].children.iter().rev() {
                    stack.push((child, false));
                }
            }
        }
    }

    pub fn draw_for_each<F>(&self, mut draw_fn: F)
    where
        F: FnMut(&ArenaNode),
    {
        let mut items = Vec::new();
        let mut stack = Vec::new();
        let mut visit_index: u32 = 0;

        for &root in self.roots.iter().rev() {
            stack.push(root);
        }

        while let Some(id) = stack.pop() {
            items.push((id, visit_index));
            visit_index += 1;

            for &child in self.inner[id].children.iter().rev() {
                stack.push(child);
            }
        }

        items.sort_unstable_by(|(a_id, a_ord), (b_id, b_ord)| {
            let za = self.inner[*a_id].node.z_index;
            let zb = self.inner[*b_id].node.z_index;
            (za, *a_ord).cmp(&(zb, *b_ord))
        });

        for (id, _) in items {
            draw_fn(&self.inner[id]);
        }
    }

    pub fn compute_layout(&mut self, rect: LogicalRect) {
        for root_id in self.roots.clone() {
            if let Some(root_node) = self.inner.get_mut(root_id) {
                root_node.rect = rect;
            }
            self.compute_layout_recursive(rect, root_id);
        }
    }
}

impl Arena {
    fn compute_layout_recursive(&mut self, rect: LogicalRect, node_id: NodeId) {
        let (children_ids, rects) = {
            let node = &self.inner[node_id];
            let children_ids = node.children.clone();
            let children = children_ids.iter().map(|&id| &self.inner[id]);
            let rects = node.node.layout.split(rect, children);
            (children_ids, rects)
        };

        for (child_id, child_rect) in children_ids.into_iter().zip(rects) {
            self.inner[child_id].rect = child_rect;
            self.compute_layout_recursive(child_rect, child_id);
        }
    }

    fn push(&mut self, node: Node, parent: Option<NodeId>) -> NodeId {
        let id = self.inner.insert(ArenaNode {
            node,
            parent,
            children: Vec::new(),
            rect: LogicalRect::new(0, 0, 0, 0),
        });

        if let Some(parent) = parent {
            self.inner[parent].children.push(id);
        } else {
            self.roots.push(id);
        }

        let children = self.inner[id]
            .node
            .iter_children()
            .map(|child| match child {
                Child::Static(factory) => factory(),
            })
            .collect::<Vec<Node>>();

        for child in children {
            self.push(child, Some(id));
        }

        id
    }
}

pub(crate) struct ArenaNode {
    node: Node,
    #[expect(unused)]
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    rect: LogicalRect,
}

impl ArenaNode {
    pub(crate) fn render(&self, buffer: &mut Buffer) {
        if let Some(renderer) = &self.node.get_renderer() {
            let mut canvas = Canvas::new(self.rect, buffer);
            renderer(&mut canvas);
        }
    }

    pub(crate) fn dispatch(&mut self, msg: &Message, ctx: &mut Context) {
        if let Some(listeners) = self.node.get_listeners(msg) {
            listeners.dispatch(msg, ctx);
        }
    }
}
