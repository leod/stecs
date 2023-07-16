use crate::EntityKey;

use super::Storage;

pub struct Iter<'a, T> {
    storage: &'a Storage<T>,
    index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (EntityKey<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
