use std::{
    cell::RefCell,
    fmt::{self, Debug},
    marker::PhantomData,
    ops::Deref,
};

use crate::{column::Column, query::fetch::Fetch, Component};

// TODO: Eq, Hash, PartialOrd, Ord.
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
        *self
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

pub trait EntityBorrow<'f> {
    type Entity: Entity;

    type Fetch<'w>: Fetch<Item<'f> = Self> + 'w
    where
        'w: 'f;

    fn new_fetch<'w>(len: usize, columns: &'w <Self::Entity as Entity>::Columns) -> Self::Fetch<'w>
    where
        'w: 'f;
}

pub trait Columns: Default + 'static {
    type Entity: Entity<Columns = Self>;

    fn column<C: Component>(&self) -> Option<&RefCell<Column<C>>>;

    fn push(&mut self, entity: Self::Entity);

    fn remove(&mut self, index: usize) -> Self::Entity;
}

pub trait Entity: Sized + 'static {
    type Columns: Columns<Entity = Self>;

    type Borrow<'f>: EntityBorrow<'f, Entity = Self>;

    type BorrowMut<'f>: EntityBorrow<'f, Entity = Self>;
}

pub struct EntityRef<'f, E: Entity>(E::Borrow<'f>);

impl<'f, E: Entity> Deref for EntityRef<'f, E> {
    type Target = E::Borrow<'f>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct EntityRefMut<'f, E: Entity>(E::BorrowMut<'f>);

impl<'f, E: Entity> Deref for EntityRefMut<'f, E> {
    type Target = E::BorrowMut<'f>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
