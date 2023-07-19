use std::slice;

#[derive(Clone, Debug)]
pub struct Column<C>(Vec<C>);

impl<C> Default for Column<C> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<C> Column<C> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, component: C) {
        self.0.push(component);
    }

    pub fn remove(&mut self, index: usize) -> C {
        assert!(index < self.0.len());

        let last = self.0.len() - 1;

        self.0.swap(index, last);
        self.0.pop().unwrap()
    }

    pub fn last(&self) -> Option<&C> {
        self.0.last()
    }

    pub fn as_raw_parts(&mut self) -> (*mut C, usize) {
        (self.0.as_mut_ptr(), self.0.len())
    }
}

// TODO: Need to allow multiple borrows by checking that entity IDs do not
// overlap.
pub type ColumnIter<'a, C> = slice::Iter<'a, C>;
