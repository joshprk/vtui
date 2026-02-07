use core::cell::{Ref, RefMut};

use generational_box::{
    GenerationalBox, GenerationalRef, GenerationalRefMut, Owner, UnsyncStorage,
};

#[derive(Default)]
pub struct StateStore {
    inner: Owner<UnsyncStorage>,
}

impl StateStore {
    pub fn insert<T: 'static>(&self, value: T) -> State<T> {
        State::new(self.inner.insert(value))
    }
}

/// Component-owned state handles which persist across listeners and can be passed to children.
///
/// State can only be accessed by the runtime thread. It is based off a reference-counted
/// generation-backed box, but never panics as invalid borrows cannot be performed.
#[derive(Debug)]
pub struct State<T: 'static> {
    inner: GenerationalBox<T>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for State<T> {}

impl<T> State<T> {
    /// Returns a reference to the inner value.
    pub fn read(&self) -> GenerationalRef<Ref<'_, T>> {
        self.inner.read()
    }

    /// Directly mutates the inner value.
    ///
    /// It is recommended to use [`State::set`] instead if you do not have a reason for using this
    /// function.
    pub fn write(&mut self) -> GenerationalRefMut<RefMut<'_, T>> {
        self.inner.write()
    }

    /// Enqueues a mutation of the inner value.
    ///
    /// Currently, this immediately mutates the state. Future releases may introduce batching
    /// behavior between siblings to prevent race conditions.
    pub fn set<F>(&mut self, write: F)
    where
        F: FnOnce(&mut T),
    {
        write(&mut self.inner.write());
    }

    pub(crate) fn new(inner: GenerationalBox<T>) -> Self {
        Self { inner }
    }
}
