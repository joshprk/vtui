use rustc_hash::FxHashSet;
use slotmap::{SlotMap, new_key_type};

use crate::component::{Identity, Ui};

new_key_type! { pub struct NodeId; }

pub struct Arena {
    root: NodeId,
    nodes: SlotMap<NodeId, Node>,
}

impl From<Node> for Arena {
    fn from(node: Node) -> Self {
        let mut nodes = SlotMap::default();
        let root = nodes.insert(node);

        Self {
            root,
            nodes,
        }
    }
}

impl Arena {
    fn reconcile(&mut self, id: NodeId) {
        let node = &self.nodes[id];
        let mut ui = Ui::default();
        (node.composer)(&mut ui);

        let new = ui.descriptors();
        let mut old = core::mem::take(&mut self.nodes[id].children);
        let mut next = Vec::with_capacity(new.len());

        let mut seen = FxHashSet::default();

        for d in new {
            assert!(seen.insert(d.identity()), "duplicate identity");

            let want = d.identity();
            let pos = old.iter().position(|&cid| {
                let have = self.nodes[cid].identity;
                have == want
            });

            let cid = if let Some(i) = pos {
                old.swap_remove(i)
            } else {
                let mut node = d.build();
                node.identity = want;
                self.nodes.insert(node)
            };

            next.push(cid);
        }

        for cid in old {
            self.unmount(cid);
        }

        self.nodes[id].children = next;
    }

    fn remount(&mut self, root: NodeId) {
        let mut stack = vec![root];

        while let Some(id) = stack.pop() {
            let mut children = self.nodes[id].children.clone();
            self.reconcile(id);
            stack.append(&mut children);
        }
    }

    fn unmount(&mut self, id: NodeId) {
        for cid in core::mem::take(&mut self.nodes[id].children) {
            self.unmount(cid);
        }

        self.nodes.remove(id);
    }
}

pub struct Node {
    pub(crate) composer: Box<dyn Fn(&mut Ui)>,

    identity: Identity,
    children: Vec<NodeId>,
}

impl Default for Node {
    #[track_caller]
    fn default() -> Self {
        Self {
            composer: Box::new(|_| {}),
            identity: Identity::unkeyed(),
            children: Vec::new(),
        }
    }
}
