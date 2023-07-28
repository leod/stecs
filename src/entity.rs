use std::{
    cell::RefCell,
    fmt::{self, Debug},
};

use crate::{
    archetype::EntityKey, column::Column, query::fetch::Fetch, Component, Query, WorldData,
};

pub trait Columns: Default + 'static {
    type Entity: Entity<Id = EntityKey<Self::Entity>> + EntityVariant<Self::Entity>;

    fn column<C: Component>(&self) -> Option<&RefCell<Column<C>>>;

    fn push(&mut self, entity: Self::Entity);

    fn remove(&mut self, index: usize) -> Self::Entity;

    // FIXME: I really don't know about these lifetimes.
    fn new_fetch<'w, 'f>(&'w self, len: usize) -> <Self::Entity as Entity>::Fetch<'f>
    where
        'w: 'f;

    // FIXME: I really don't know about these lifetimes.
    fn new_fetch_mut<'w, 'f>(&'w self, len: usize) -> <Self::Entity as Entity>::FetchMut<'f>
    where
        'w: 'f;
}

pub trait Entity: Sized + 'static {
    type Id: Copy + Debug + PartialEq + 'static;

    type Ref<'f>: Query;
    /*where
    for<'w> <Self::Ref<'w> as Query>::Fetch<'w>: Fetch<Item<'w> = Self::Ref<'w>>;*/

    type RefMut<'f>: Query;

    #[doc(hidden)]
    type Fetch<'w>: Fetch<Item<'w> = Self::Ref<'w>>;

    #[doc(hidden)]
    type FetchMut<'w>: Fetch<Item<'w> = Self::RefMut<'w>>;

    type FetchId<'w>: Fetch<Item<'w> = Self::Id>;

    type WorldData: WorldData<Entity = Self>;
}

pub trait EntityStruct: Entity {
    type Columns: Columns<Entity = Self>;
}

pub trait EntityVariant<EOuter: Entity>: Entity {
    fn into_outer(self) -> EOuter;

    fn id_to_outer(id: Self::Id) -> EOuter::Id;
}

pub type EntityRef<'f, E> = <E as Entity>::Ref<'f>;

pub type EntityRefMut<'f, E> = <E as Entity>::RefMut<'f>;

pub struct EntityId<E: Entity>(E::Id);

impl<E: Entity> Clone for EntityId<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: Entity> Copy for EntityId<E> {}

impl<E: Entity> Debug for EntityId<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EntityId").field(&self.0).finish()
    }
}

impl<E: Entity> PartialEq for EntityId<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<E: Entity> EntityId<E> {
    pub fn new_unchecked(id: E::Id) -> Self {
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
