use crate::component::Node;

#[derive(Default)]
pub(crate) struct Arena {
    arena: Vec<Node>,
}

impl Arena {
    pub fn new(root: Node) -> Self {
        let mut arena = Self::default();
        arena.push(root);
        arena
    }

    pub fn iter_draw(&self) -> impl Iterator<Item = &Node> {
        self.arena.iter()
    }

    pub fn iter_update(&mut self) -> impl Iterator<Item = &mut Node> {
        self.arena.iter_mut()
    }
}

impl Arena {
    fn push(&mut self, node: Node) {
        self.arena.push(node);
    }
}
