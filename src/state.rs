use std::cell::{Ref, RefMut};

use generational_box::{
    GenerationalBox, GenerationalRef, GenerationalRefMut, Owner, UnsyncStorage,
};

#[derive(Default)]
pub(crate) struct StateOwner {
    inner: Owner<UnsyncStorage>,
}

impl StateOwner {
    pub fn insert<T: 'static>(&self, value: T) -> State<T> {
        let inner = self.inner.insert(value);
        State { inner }
    }
}

#[derive(Debug)]
pub struct State<T> {
    inner: GenerationalBox<T>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for State<T> {}

impl<T: 'static> State<T> {
    pub fn read(&self) -> GenerationalRef<Ref<'_, T>> {
        self.inner.read()
    }

    pub fn write(&mut self) -> GenerationalRefMut<RefMut<'_, T>> {
        self.inner.write()
    }
}
