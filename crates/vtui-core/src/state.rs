use std::cell::{Ref, RefMut};

use generational_box::{
    AnyStorage, GenerationalBox, GenerationalRef, GenerationalRefMut, Owner, UnsyncStorage,
};

#[derive(Default)]
pub(crate) struct StateOwner {
    inner: Owner<UnsyncStorage>,
}

impl StateOwner {
    pub fn new() -> Self {
        let inner = UnsyncStorage::owner();
        Self { inner }
    }

    pub fn insert<T: 'static>(&self, value: T) -> State<T> {
        let inner = self.inner.insert(value);
        State { inner }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct State<T> {
    inner: GenerationalBox<T>,
}

impl<T: 'static> State<T> {
    pub fn read(&self) -> GenerationalRef<Ref<'_, T>> {
        self.inner.read()
    }

    pub fn write(&mut self) -> GenerationalRefMut<RefMut<'_, T>> {
        self.inner.write()
    }
}
