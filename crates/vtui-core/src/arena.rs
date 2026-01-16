use std::ops::{Index, IndexMut};

use slotmap::{SlotMap, new_key_type};

use crate::component::{Child, Node};

new_key_type! { pub struct NodeId; }

#[derive(Default)]
pub(crate) struct Arena {
    roots: Vec<NodeId>,
    inner: SlotMap<NodeId, ArenaNode>,
}

impl Index<NodeId> for Arena {
    type Output = Node;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.inner[index].node
    }
}

impl IndexMut<NodeId> for Arena {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.inner[index].node
    }
}

impl Arena {
    pub fn new(root: Node) -> Self {
        let mut arena = Self::default();
        arena.push(root, None);
        arena
    }

    pub fn iter_draw(&self) -> impl Iterator<Item = NodeId> {
        let mut order = Vec::new();
        let mut stack = Vec::new();

        for &root in self.roots.iter().rev() {
            stack.push(root);
        }

        while let Some(id) = stack.pop() {
            order.push(id);

            let children = &self.inner[id].children;
            for &child in children.iter().rev() {
                stack.push(child);
            }
        }

        order.sort_by_key(|&id| self.inner[id].node.z_index);
        order.into_iter()
    }

    pub fn iter_update(&mut self) -> impl Iterator<Item = NodeId> + use<> {
        let mut out = Vec::new();
        let mut stack = Vec::new();

        for &root in self.roots.iter().rev() {
            stack.push((root, false));
        }

        while let Some((id, visited)) = stack.pop() {
            if visited {
                out.push(id);
            } else {
                stack.push((id, true));
                for &child in self.inner[id].children.iter().rev() {
                    stack.push((child, false));
                }
            }
        }

        out.into_iter()
    }
}

impl Arena {
    fn push(&mut self, node: Node, parent: Option<NodeId>) -> NodeId {
        let id = self.inner.insert(ArenaNode {
            node,
            parent,
            children: Vec::new(),
        });

        if let Some(parent) = parent {
            self.inner[parent].children.push(id);
        } else {
            self.roots.push(id);
        }

        let children = self.inner[id].node
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

struct ArenaNode {
    node: Node,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}
