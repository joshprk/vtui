use std::ops::{Index, IndexMut};

use crate::component::Component;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ComponentId(usize);

impl ComponentId {
    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}

impl From<usize> for ComponentId {
    fn from(value: usize) -> Self {
        ComponentId(value)
    }
}

pub(crate) struct Arena {
    root: ComponentId,
    inner: Vec<ComponentNode>,
}

impl Index<ComponentId> for Arena {
    type Output = ComponentNode;

    fn index(&self, index: ComponentId) -> &Self::Output {
        &self.inner[index.index()]
    }
}

impl IndexMut<ComponentId> for Arena {
    fn index_mut(&mut self, index: ComponentId) -> &mut Self::Output {
        &mut self.inner[index.index()]
    }
}

impl Arena {
    pub fn new(root: Component) -> Self {
        let mut arena = Self {
            root: ComponentId(0),
            inner: Vec::default(),
        };

        arena.push(root, None);
        arena
    }

    pub fn push(&mut self, component: Component, parent: Option<ComponentId>) {
        let id = ComponentId(self.inner.len());

        let node = ComponentNode {
            component,
            parent,
            children: Vec::default(),
        };

        self.inner.push(node);

        let children = self[id]
            .component
            .inner()
            .iter_child()
            .map(|c| c.mount())
            .collect::<Vec<Component>>();

        for child in children {
            let next_id = ComponentId(self.inner.len());
            self.push(child, Some(id));
            self[id].children.push(next_id);
        }
    }

    pub fn iter_draw(&self) -> impl Iterator<Item = &Component> {
        self.inner.iter().map(|n| &n.component)
    }

    pub fn iter_update(&mut self) -> impl Iterator<Item = &mut Component> {
        self.inner.iter_mut().map(|n| &mut n.component)
    }
}

pub(crate) struct ComponentNode {
    pub component: Component,
    pub parent: Option<ComponentId>,
    pub children: Vec<ComponentId>,
}
