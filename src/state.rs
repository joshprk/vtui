use alloc::rc::Rc;
use core::cell::RefCell;

#[derive(Clone)]
pub struct Callback {
    inner: Rc<RefCell<dyn FnMut()>>,
}

impl PartialEq for Callback {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<F: FnMut() + 'static> From<F> for Callback {
    fn from(value: F) -> Self {
        let inner = Rc::new(RefCell::new(value));
        Self { inner }
    }
}

impl Callback {
    pub fn call(&self) {
        self.inner.borrow_mut()()
    }
}
