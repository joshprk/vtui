use std::ops::{Index, IndexMut};

use slotmap::{SlotMap, new_key_type};

use crate::{canvas::LogicalRect, component::{Child, Node}};

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
            computed_rect: None,
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

impl Arena {
    pub fn get_computed_layout(&self, node_id: NodeId) -> Option<LogicalRect> {
        self.inner.get(node_id)?.computed_rect
    }

    pub fn compute_layout(&mut self, terminal_area: LogicalRect) {
        // Phase 1: Reset computed areas (mutable borrow)
        for node in self.inner.values_mut() {
            node.computed_rect = None;
        }
        
        // Phase 2: Compute layout (separate mutable borrow)
        for root_id in self.roots.clone() {
            if let Some(root_node) = self.inner.get_mut(root_id) {
                root_node.computed_rect = Some(terminal_area);
            }
            self.compute_node_layout_recursive(root_id, terminal_area);
        }
    }
    
    fn compute_node_layout_recursive(&mut self, node_id: NodeId, available_area: LogicalRect) {
        // Take node data separately to avoid borrow conflicts
        let (parent_layout, child_ids) = {
            let arena_node = &self.inner[node_id];
            (arena_node.node.layout, arena_node.children.clone())
        };
        
        // Compute layout using node_data
        let child_areas = {
            let children: Vec<&Node> = child_ids
                .iter()
                .map(|&id| &self.inner[id].node)
                .collect();
            parent_layout.split(available_area, children)
        };

        for (&child_id, &area) in child_ids.iter().zip(child_areas.iter()) {
            let child_node = self.inner.get_mut(child_id).unwrap();
            child_node.computed_rect = Some(area);
            self.compute_node_layout_recursive(child_id, area);
        }
    }
}

struct ArenaNode {
    node: Node,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    computed_rect: Option<LogicalRect>,
}
