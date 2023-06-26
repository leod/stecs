use thunderdome::Arena;

use crate::Component;

pub(crate) struct Column<C>(Arena<C>);

pub(crate) type ColumnIter<'a, C> = thunderdome::iter::Iter<'a, C>;

pub trait Archetype {
    type Storage;

    fn has<C: Component>() -> bool;

    fn column<C: Component>(storage: &Self::Storage) -> Option<Column<C>>;
}

pub type Storage<A> = <A as Archetype>::Storage;
