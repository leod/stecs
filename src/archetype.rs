use std::{
    cell::RefCell,
    fmt::{self, Debug},
    marker::PhantomData,
};

use thunderdome::Arena;

use crate::{column::Column, Component};

// TODO: PartialEq, Eq, Hash, PartialOrd, Ord.
// https://github.com/rust-lang/rust/issues/26925
pub struct EntityKey<E>(pub thunderdome::Index, PhantomData<E>);

impl<E> EntityKey<E> {
    #[doc(hidden)]
    pub fn new_unchecked(untyped_key: thunderdome::Index) -> Self {
        Self(untyped_key, PhantomData)
    }
}

impl<E> Clone for EntityKey<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<E> Copy for EntityKey<E> {}

impl<E> Debug for EntityKey<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EntityKey").field(&self.0).finish()
    }
}

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
    untyped_keys: Column<thunderdome::Index>,
    columns: E::Columns,
}

impl<E: Entity> Archetype<E> {
    pub fn indices(&self) -> &Arena<usize> {
        &self.indices
    }

    pub fn untyped_keys(&self) -> &Column<thunderdome::Index> {
        &self.untyped_keys
    }

    pub fn columns(&self) -> &E::Columns {
        &self.columns
    }

    pub fn spawn(&mut self, entity: E) -> EntityKey<E> {
        let index = self.untyped_keys.len();
        let key = EntityKey::new_unchecked(self.indices.insert(index));

        self.untyped_keys.push(key.0);
        self.columns.push(entity);

        key
    }

    pub fn despawn(&mut self, key: EntityKey<E>) -> Option<E> {
        let index = self.indices.remove(key.0)?;

        self.untyped_keys.remove(index);

        if let Some(last) = self.untyped_keys.last() {
            self.indices[*last] = self.untyped_keys.len() - 1;
        }

        Some(self.columns.remove(index))
    }
}

impl<E: Entity> Default for Archetype<E> {
    fn default() -> Self {
        Self {
            indices: Default::default(),
            untyped_keys: Default::default(),
            columns: Default::default(),
        }
    }
}
