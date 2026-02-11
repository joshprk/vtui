use core::cell::{Ref, RefMut};

use generational_box::{
    GenerationalBox, GenerationalRef, GenerationalRefMut, Owner, UnsyncStorage,
};

use crate::errors::StateError;

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
    /// Attempts to return a reference to the value, returning an error if it is freed.
    pub fn try_read(&self) -> Result<GenerationalRef<Ref<'_, T>>, StateError> {
        self.inner.try_read().map_err(StateError::from)
    }

    /// Attempts to return a mutable reference to the value, returning an error if it is freed.
    pub fn try_write(&self) -> Result<GenerationalRefMut<RefMut<'_, T>>, StateError> {
        self.inner.try_write().map_err(StateError::from)
    }

    /// Returns a reference to the value.
    ///
    /// # Panics
    ///
    /// Panics if the component which this state was created from is freed.
    ///
    /// This should never happen if state is not self-referential (i.e. `State<State<T>>`).
    pub fn read(&self) -> GenerationalRef<Ref<'_, T>> {
        self.try_read().expect("attempted to read state after free")
    }

    /// Directly mutates the value.
    ///
    /// It is recommended to use [`State::set`] instead if you do not have a reason for using this
    /// function.
    ///
    /// # Panics
    ///
    /// Panics if the component which this state was created from is freed.
    ///
    /// This should never happen if state is not self-referential (i.e. `State<State<T>>`).
    pub fn write(&mut self) -> GenerationalRefMut<RefMut<'_, T>> {
        self.try_write().expect("attempted to read state after free")
    }

    /// Enqueues a mutation of the value.
    ///
    /// Currently, this immediately mutates the state. Future releases may introduce batching
    /// behavior between siblings to prevent race conditions.
    ///
    /// # Panics
    ///
    /// Panics if the component which this state was created from is freed.
    ///
    /// This should never happen if state is not self-referential (i.e. `State<State<T>>`).
    pub fn set<F>(&mut self, write: F)
    where
        F: FnOnce(&mut T),
    {
        write(&mut self.write());
    }

    pub(crate) fn new(inner: GenerationalBox<T>) -> Self {
        Self { inner }
    }
}
