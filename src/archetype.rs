use std::cell::RefCell;

use frunk::prelude::HList;
use thunderdome::{Arena, Index};

use crate::Component;

pub type EntityIndex = thunderdome::Index;

pub trait Archetype {
    type Components: HList;
    type Storage: Default;

    fn column<'a, C: Component>(storage: &'a Self::Storage) -> Option<&'a Column<C>>;

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

    pub fn iter(&self) -> ColumnIter<C> {
        self.0.iter()
    }
}

pub type ColumnIter<'a, C> = thunderdome::iter::Iter<'a, C>;

pub struct ColumnValues<'a, C>(pub(crate) ColumnIter<'a, C>);

impl<'a, C> Iterator for ColumnValues<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, value)| value)
    }
}

pub struct Storage<S: Archetype>(S::Storage);

impl<A: Archetype> Storage<A> {
    pub fn insert(&mut self, entity: A) -> Index {
        A::insert(&mut self.0, entity)
    }

    pub fn column<'a, C: Component>(&'a self) -> Option<&'a Column<C>> {
        A::column(&self.0)
    }
}

impl<A: Archetype> Default for Storage<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}
