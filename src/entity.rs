use std::{any::Any, fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::{
    archetype::EntityKey, column::Column, query::fetch::Fetch, Component, Query, QueryShared,
    WorldData,
};

pub trait Columns: Default + Clone + 'static {
    type Entity: Entity<Id = EntityKey<Self::Entity>> + EntityVariant<Self::Entity>;

    fn column<C: Component>(&self) -> Option<&Column<C>>;

    fn push(&mut self, entity: Self::Entity);

    fn remove(&mut self, index: usize) -> Self::Entity;

    #[doc(hidden)]
    fn new_fetch(&self, len: usize) -> <Self::Entity as Entity>::Fetch<'_>;

    #[doc(hidden)]
    fn new_fetch_mut(&self, len: usize) -> <Self::Entity as Entity>::FetchMut<'_>;
}

pub trait Entity: Clone + 'static {
    type Id: Copy + Debug + Eq + Ord + Hash + 'static;

    type Ref<'a>: QueryShared + Clone;
    /*where
    for<'w> <Self::Ref<'w> as Query>::Fetch<'w>: Fetch<Item<'w> = Self::Ref<'w>>;*/

    type RefMut<'a>: Query;

    type WorldData: WorldData<Entity = Self>;

    #[doc(hidden)]
    type FetchId<'w>: Fetch<Item<'w> = EntityId<Self>> + 'w;

    #[doc(hidden)]
    type Fetch<'w>: Fetch<Item<'w> = Self::Ref<'w>> + 'w;

    #[doc(hidden)]
    type FetchMut<'w>: Fetch<Item<'w> = Self::RefMut<'w>> + 'w;

    fn from_ref<'a>(entity: Self::Ref<'a>) -> Self;
}

pub trait EntityStruct: Entity {
    type Columns: Columns<Entity = Self>;
}

pub trait EntityVariant<EOuter: Entity>: Entity {
    fn into_outer(self) -> EOuter;

    fn id_to_outer(id: Self::Id) -> EOuter::Id;
}

pub type EntityRef<'a, E> = <E as Entity>::Ref<'a>;

pub type EntityRefMut<'a, E> = <E as Entity>::RefMut<'a>;

#[derive(Derivative)]
#[derivative(
    Copy(bound = ""),
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = ""),
    Hash(bound = ""),
    Debug(bound = "")
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityId<E: Entity>(E::Id);

impl<E: Entity> EntityId<E> {
    pub fn new(id: E::Id) -> Self {
        Self(id)
    }

    pub fn from<EInner: EntityVariant<E>>(id: EntityId<EInner>) -> Self {
        id.to_outer()
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

// For proc macros.
#[doc(hidden)]
pub fn downcast_columns_ref<T: Columns, U: Columns>(column: &T) -> Option<&U> {
    (column as &dyn Any).downcast_ref()
}
