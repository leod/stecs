use std::{
    cell::RefCell,
    fmt::{self, Debug},
    marker::PhantomData,
};

use thunderdome::Arena;

use crate::{
    column::Column,
    query::fetch::{Fetch, FetchFromSet},
    ArchetypeSet, Component,
};

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

impl<E> PartialEq for EntityKey<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub trait EntityColumns: Default {
    type Entity: Entity<Columns = Self>;

    fn column<C: Component>(&self) -> Option<&RefCell<Column<C>>>;

    fn push(&mut self, entity: Self::Entity);

    fn remove(&mut self, index: usize) -> Self::Entity;
}

pub trait BorrowEntity<'f> {
    type Entity: Entity;

    type Fetch<'w>: Fetch<'w, Item<'f> = Self>
    where
        'w: 'f;

    fn to_entity(&'f self) -> Self::Entity;

    fn new_fetch<'w>(columns: &'w <Self::Entity as Entity>::Columns) -> Self::Fetch<'w>
    where
        'w: 'f;
}

pub trait Entity: Sized {
    type BorrowMut<'f>: BorrowEntity<'f, Entity = Self>;

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

    pub fn get_mut(&mut self, key: EntityKey<E>) -> Option<E::BorrowMut<'_>> {
        let index = *self.indices.get(key.0)?;

        debug_assert!(index < self.untyped_keys.len());

        let fetch = <E::BorrowMut<'_> as BorrowEntity<'_>>::new_fetch(&self.columns);

        // Safety: TODO
        Some(unsafe { fetch.get(index) })
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
