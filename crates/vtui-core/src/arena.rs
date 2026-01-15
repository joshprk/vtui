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
    inner: Vec<Component>,
    parents: Vec<ComponentId>,
}

impl Index<ComponentId> for Arena {
    type Output = Component;

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
            inner: Vec::default(),
            parents: Vec::default(),
        };

        arena.push(root, None);
        arena
    }

    pub fn push(&mut self, component: Component, parent: Option<ComponentId>) {
        let id = ComponentId(self.inner.len());
        let parent_id = match parent {
            Some(parent_id) => parent_id,
            None => id,
        };

        self.inner.push(component);
        self.parents.push(parent_id);

        let children = self[id]
            .inner()
            .iter_child()
            .map(|c| c.mount())
            .collect::<Vec<Component>>();

        for child in children {
            self.push(child, Some(id));
        }
    }

    pub fn iter_draw(&self) -> impl Iterator<Item = &Component> {
        self.inner.iter()
    }

    pub fn iter_update(&mut self) -> impl Iterator<Item = &mut Component> {
        self.inner.iter_mut()
    }
}
