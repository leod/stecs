use std::{cell::RefCell, marker::PhantomData};

pub use thunderdome::Arena;

use crate::{column::Column, Component};

// TODO: Debug, PartialEq, Eq, Hash, PartialOrd, Ord.
// https://github.com/rust-lang/rust/issues/26925
pub struct EntityKey<E>(pub thunderdome::Index, PhantomData<E>);

impl<E> Clone for EntityKey<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<E> Copy for EntityKey<E> {}

pub trait EntityColumns: Default {
    type Entity: Entity<Columns = Self>;

    fn column<C: Component>(&self) -> Option<&RefCell<Column<C>>>;

    fn push(&mut self, entity: Self::Entity);

    fn remove(&mut self, index: usize) -> Self::Entity;
}

pub trait Entity: Sized {
    type Columns: EntityColumns<Entity = Self>;
}

#[derive(Clone)]
pub struct Archetype<E: Entity> {
    indices: Arena<usize>,
    ids: Column<EntityKey<E>>,
    columns: E::Columns,
}

impl<E: Entity> Archetype<E> {
    pub(crate) fn column<C: Component>(&self) -> Option<&RefCell<Column<C>>> {
        self.columns.column::<C>()
    }

    pub fn indices(&self) -> &Arena<usize> {
        &self.indices
    }

    pub fn columns(&self) -> &E::Columns {
        &self.columns
    }

    pub fn spawn(&mut self, entity: E) -> EntityKey<E> {
        let index = self.ids.len();
        let key = EntityKey(self.indices.insert(index), PhantomData);

        self.ids.push(key);
        self.columns.push(entity);

        key
    }

    pub fn despawn(&mut self, key: EntityKey<E>) -> Option<E> {
        let index = self.indices.remove(key.0)?;

        self.ids.remove(index);

        if let Some(last) = self.ids.last() {
            self.indices[last.0] = self.ids.len() - 1;
        }

        Some(self.columns.remove(index))
    }
}

impl<E: Entity> Default for Archetype<E> {
    fn default() -> Self {
        Self {
            indices: Default::default(),
            ids: Default::default(),
            columns: Default::default(),
        }
    }
}
