use std::{
    cell::RefCell,
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{archetype::EntityKey, column::Column, query::fetch::Fetch, Component, WorldData};

/*
// TODO: Eq, Hash, PartialOrd, Ord.
// https://github.com/rust-lang/rust/issues/26925
pub struct EntityId<E>(pub thunderdome::Index, PhantomData<E>);

impl<E> EntityId<E> {
    #[doc(hidden)]
    pub fn new_unchecked(id: thunderdome::Index) -> Self {
        Self(id, PhantomData)
    }
}

impl<E> Clone for EntityId<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for EntityId<E> {}

impl<E> Debug for EntityId<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EntityId").field(&self.0).finish()
    }
}

impl<E> PartialEq for EntityId<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
*/

pub trait EntityBorrow<'f> {
    type Entity: Entity;
}

pub trait Columns: Default + 'static {
    type Entity: Entity<Id = EntityKey<Self::Entity>> + EntityVariant<Self::Entity>;

    type Fetch<'w>: Fetch<Item<'w> = <Self::Entity as Entity>::Ref<'w>>;

    type FetchMut<'w>: Fetch<Item<'w> = <Self::Entity as Entity>::RefMut<'w>>;

    fn column<C: Component>(&self) -> Option<&RefCell<Column<C>>>;

    fn push(&mut self, entity: Self::Entity);

    fn remove(&mut self, index: usize) -> Self::Entity;

    // FIXME: I really don't know about these lifetimes.
    fn new_fetch<'w, 'f>(&'w self, len: usize) -> Self::Fetch<'f>
    where
        'w: 'f;

    // FIXME: I really don't know about these lifetimes.
    fn new_fetch_mut<'w, 'f>(&'w self, len: usize) -> Self::FetchMut<'f>
    where
        'w: 'f;
}

pub trait Entity: Sized + 'static {
    type Id: Copy + PartialEq + 'static;

    type Ref<'f>: EntityBorrow<'f, Entity = Self>;

    type RefMut<'f>: EntityBorrow<'f, Entity = Self>;

    type WorldData: WorldData<Entity = Self>;
}

pub trait ConcreteEntity: Entity {
    type Columns: Columns<Entity = Self>;
}

pub trait EntityVariant<EOuter: Entity>: Entity {
    fn into_outer(self) -> EOuter;

    fn id_to_outer(id: Self::Id) -> EOuter::Id;
}

pub struct EntityRef<'f, E: Entity>(pub(crate) E::Ref<'f>);

impl<'f, E: Entity> Deref for EntityRef<'f, E> {
    type Target = E::Ref<'f>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct EntityRefMut<'f, E: Entity>(pub(crate) E::RefMut<'f>);

impl<'f, E: Entity> Deref for EntityRefMut<'f, E> {
    type Target = E::RefMut<'f>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'f, E: Entity> DerefMut for EntityRefMut<'f, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct EntityId<E: Entity>(E::Id);

impl<E: Entity> Clone for EntityId<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: Entity> Copy for EntityId<E> {}

impl<E: Entity> PartialEq for EntityId<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<E: Entity> EntityId<E> {
    pub(crate) fn new(id: E::Id) -> Self {
        Self(id)
    }

    pub fn get(self) -> E::Id {
        self.0
    }

    pub fn to_outer<EOuter>(self) -> EntityId<EOuter>
    where
        EOuter: Entity,
        E: EntityVariant<EOuter>,
    {
        EntityId(E::id_to_outer(self.0))
    }
}
