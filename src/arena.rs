use rustc_hash::FxHashSet;
use slotmap::{SlotMap, new_key_type};

use crate::{
    component::{AnyProps, Identity, Ui},
    listeners::Listeners,
};

new_key_type! { pub struct NodeId; }

pub struct Arena {
    root: NodeId,
    nodes: SlotMap<NodeId, Node>,
}

impl From<Node> for Arena {
    fn from(node: Node) -> Self {
        let mut nodes = SlotMap::default();
        let root = nodes.insert(node);

        let mut arena = Self { root, nodes };
        arena.reconcile(root);
        arena
    }
}

impl Arena {
    fn reconcile(&mut self, id: NodeId) {
        let node = &self.nodes[id];
        let mut ui = Ui::default();
        (node.composer)(&mut ui);

        let new = ui.into_descriptors();
        let mut old = core::mem::take(&mut self.nodes[id].children);
        let mut next = Vec::with_capacity(new.len());
        let mut seen = FxHashSet::default();

        for d in new {
            let identity = d.identity();

            assert!(seen.insert(identity), "duplicate identity");

            let pos = old
                .iter()
                .position(|&cid| self.nodes[cid].identity == identity);
            let cid = if let Some(i) = pos {
                let cid = old.swap_remove(i);
                if !self.nodes[cid].props.eq(d.props()) {
                    self.unmount_children(cid);
                    self.nodes[cid] = d.build();
                }
                cid
            } else {
                self.nodes.insert(d.build())
            };

            next.push(cid);
        }

        for cid in old {
            self.unmount(cid);
        }

        self.nodes[id].children = next;

        for &cid in &self.nodes[id].children.clone() {
            self.reconcile(cid);
        }
    }

    fn unmount_children(&mut self, id: NodeId) {
        for cid in core::mem::take(&mut self.nodes[id].children) {
            self.unmount(cid);
        }
    }

    fn unmount(&mut self, id: NodeId) {
        self.unmount_children(id);
        self.nodes.remove(id);
    }
}

pub struct Node {
    pub(crate) composer: Box<dyn Fn(&mut Ui)>,
    pub(crate) props: Box<dyn AnyProps>,
    pub(crate) identity: Identity,
    pub(crate) listeners: Listeners,

    children: Vec<NodeId>,
}

impl Default for Node {
    #[track_caller]
    fn default() -> Self {
        Self {
            composer: Box::new(|_| {}),
            props: Box::new(()),
            identity: Identity::unkeyed(),
            listeners: Listeners::default(),
            children: Vec::new(),
        }
    }
}
