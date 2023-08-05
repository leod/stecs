use std::{any::Any, cell::UnsafeCell};

use crate::Component;

#[derive(Debug)]
pub struct Column<C>(UnsafeCell<Vec<C>>);

unsafe impl<C: Send> Send for Column<C> {}
unsafe impl<C: Sync> Sync for Column<C> {}

impl<C: Clone> Clone for Column<C> {
    fn clone(&self) -> Self {
        Self(UnsafeCell::new(self.borrow().clone()))
    }
}

impl<C> Default for Column<C> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<C> Column<C> {
    fn borrow(&self) -> &Vec<C> {
        let ptr = self.0.get();

        // Safety: We use the `UnsafeCell` only internally. We do expose
        // `ColumnRawParts` and `ColumnRawPartsMut`, which contain pointers to
        // our data, but the rest of the library, specifically places that
        // provide a `QueryBorrow`, and thereby a safe interface, ensure that no
        // disallowed aliasing happens.
        unsafe { &*ptr }
    }

    fn borrow_mut(&mut self) -> &mut Vec<C> {
        let ptr = self.0.get();

        // Safety: See `borrow`.
        unsafe { &mut *ptr }
    }

    pub fn len(&self) -> usize {
        self.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> &C {
        &self.borrow()[index]
    }

    pub fn push(&mut self, component: C) {
        self.borrow_mut().push(component);
    }

    pub fn remove(&mut self, index: usize) -> C {
        let inner = self.borrow_mut();

        assert!(index < inner.len());

        let last = inner.len() - 1;

        inner.swap(index, last);
        inner.pop().unwrap()
    }

    pub fn last(&self) -> Option<&C> {
        self.borrow().last()
    }

    pub fn as_slice(&self) -> &[C] {
        self.borrow().as_slice()
    }

    pub fn into_vec(self) -> Vec<C> {
        self.0.into_inner()
    }

    // TODO: Make pub(crate)
    pub fn as_raw_parts(&self) -> ColumnRawParts<C> {
        ColumnRawParts {
            ptr: self.borrow().as_ptr(),
            len: self.borrow().len(),
        }
    }

    // TODO: Make pub(crate)
    pub fn as_raw_parts_mut(&self) -> ColumnRawPartsMut<C> {
        // Safety: See `borrow`.
        let inner = unsafe { &mut *self.0.get() };

        ColumnRawPartsMut {
            ptr: inner.as_mut_ptr(),
            len: inner.len(),
        }
    }
}

pub struct ColumnRawParts<C> {
    pub ptr: *const C,
    pub len: usize,
}

impl<C> Clone for ColumnRawParts<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for ColumnRawParts<C> {}

pub struct ColumnRawPartsMut<C> {
    pub ptr: *mut C,
    pub len: usize,
}

impl<C> Clone for ColumnRawPartsMut<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for ColumnRawPartsMut<C> {}

// For proc macros.
#[doc(hidden)]
pub fn downcast_ref<C: Component, D: Component>(column: &Column<C>) -> Option<&Column<D>> {
    (column as &dyn Any).downcast_ref()
}
