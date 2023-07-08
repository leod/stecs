use frunk::prelude::HList;
use thunderdome::{Arena, Index};

use crate::Component;

pub type EntityIndex = thunderdome::Index;

pub trait Archetype {
    type Components: HList;
    type Storage: Default;

    fn has<C: Component>() -> bool;

    fn column<C: Component>(storage: &Self::Storage) -> Option<Column<C>>;

    fn insert(storage: &mut Self::Storage, entity: Self) -> EntityIndex;
}

#[derive(Debug, Clone)]
pub struct Column<C>(Arena<C>);

impl<C> Default for Column<C> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<C: Component> Column<C> {
    pub fn insert(&mut self, component: C) -> Index {
        self.0.insert(component)
    }
}

pub type ColumnIter<'a, C> = thunderdome::iter::Iter<'a, C>;

pub struct Storage<S: Archetype>(S::Storage);

impl<A: Archetype> Storage<A> {
    pub fn insert(&mut self, entity: A) -> Index {
        A::insert(&mut self.0, entity)
    }
}

impl<A: Archetype> Default for Storage<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}
