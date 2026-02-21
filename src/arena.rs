use alloc::vec::IntoIter;

use ratatui::Frame;
use rustc_hash::FxHashSet;
use slotmap::{SlotMap, new_key_type};

use crate::{
    canvas::Canvas,
    component::{AnyProps, Descriptor, Identity, Ui},
    context::Context,
    handler::EventHandler,
    layout::Region,
    listeners::Listeners,
    transport::Event,
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
    pub fn update<E: Event>(&mut self, event: &E, context: &mut Context) {
        for id in self.compute_traversal().into_iter().rev() {
            let data = self.frame_data(id);
            let handler = EventHandler::new(event, context, data);
            self.nodes[id].update(handler);
        }

        self.reconcile(self.root);
    }

    pub fn render(&mut self, frame: &mut Frame, context: &Context) {
        for id in self.compute_traversal().into_iter() {
            let data = self.frame_data(id);
            let canvas = Canvas::new(frame.buffer_mut(), context, data);
            self.nodes[id].render(canvas);
        }
    }

    fn compute_traversal(&self) -> Vec<NodeId> {
        let mut order = Vec::new();
        let mut stack = vec![self.root];

        while let Some(id) = stack.pop() {
            let node = &self.nodes[id];
            order.push(id);
            stack.extend(&node.children);
        }

        stack
    }

    fn frame_data(&self, id: NodeId) -> FrameData {
        let node = &self.nodes[id];

        FrameData {
            current_node: id,
            rect: node.rect,
        }
    }

    fn reconcile(&mut self, id: NodeId) {
        let mut old = core::mem::take(&mut self.nodes[id].children);
        let mut seen = FxHashSet::default();

        self.nodes[id].children = self.nodes[id]
            .compose()
            .map(|desc| {
                assert!(
                    seen.insert(desc.identity()),
                    "ambiguous composition detected",
                );

                let mut old_ids = old.iter().map(|&id| &self.nodes[id].identity);
                let pos = old_ids.position(|&old_id| old_id == desc.identity());

                if let Some(pos) = pos
                    && !self.needs_remount(old[pos], &desc)
                {
                    old.remove(pos)
                } else {
                    self.nodes.insert(desc.build())
                }
            })
            .collect();

        for old_id in old {
            self.remove_subtree(old_id);
        }
    }

    fn needs_remount(&self, id: NodeId, descriptor: &Descriptor) -> bool {
        let id_matches = self.nodes[id].identity == descriptor.identity();
        let props_matches = self.nodes[id].props.eq(descriptor.props());
        !(id_matches && props_matches)
    }

    fn remove_subtree(&mut self, id: NodeId) {
        for cid in core::mem::take(&mut self.nodes[id].children) {
            self.remove_subtree(cid);
        }

        self.nodes.remove(id);
    }
}

pub struct Node {
    pub(crate) composer: Box<dyn Fn(&mut Ui)>,
    pub(crate) props: Box<dyn AnyProps>,
    pub(crate) identity: Identity,
    pub(crate) renderer: Box<dyn Fn(&mut Canvas)>,
    pub(crate) listeners: Listeners,

    children: Vec<NodeId>,
    rect: Region,
}

impl Node {
    fn compose(&self) -> IntoIter<Descriptor> {
        let mut ui = Ui::default();
        let composer = &self.composer;
        composer(&mut ui);

        ui.into_descriptors()
    }

    fn render(&self, mut canvas: Canvas) {
        let renderer = &self.renderer;
        renderer(&mut canvas);
    }

    fn update<E: Event>(&mut self, mut handler: EventHandler<E>) {
        self.listeners.dispatch(&mut handler);
    }
}

impl Default for Node {
    #[track_caller]
    fn default() -> Self {
        Self {
            composer: Box::new(|_| {}),
            props: Box::new(()),
            identity: Identity::unkeyed(),
            listeners: Listeners::default(),
            renderer: Box::new(|_| {}),
            children: Vec::new(),
            rect: Region::zeroed(),
        }
    }
}

pub struct FrameData {
    pub(crate) current_node: NodeId,
    pub(crate) rect: Region,
}
