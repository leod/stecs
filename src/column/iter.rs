use super::{Column, ColumnKey};

pub struct Iter<'a, C> {
    column: &'a Column<C>,
    index: usize,
}

impl<'a, C> Iterator for Iter<'a, C> {
    type Item = (ColumnKey, &'a C);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
