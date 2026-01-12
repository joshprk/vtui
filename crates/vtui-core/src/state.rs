use std::cell::{Ref, RefMut};

use generational_box::{GenerationalBox, GenerationalRef, GenerationalRefMut};

#[derive(Clone, Copy, Debug)]
pub struct State<T> {
    inner: GenerationalBox<T>,
}

impl<T: 'static> State<T> {
    pub(crate) fn new(inner: GenerationalBox<T>) -> Self {
        State { inner }
    }

    pub fn read(&self) -> GenerationalRef<Ref<'_, T>> {
        self.inner.read()
    }

    pub fn write(&mut self) -> GenerationalRefMut<RefMut<'_, T>> {
        self.inner.write()
    }
}
