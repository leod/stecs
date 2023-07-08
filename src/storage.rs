use thunderdome::Arena;

use crate::Archetype;

pub(crate) struct Column<C>(Arena<C>);

pub(crate) type ColumnIter<'a, C> = thunderdome::iter::Iter<'a, C>;

pub trait Storage {}
