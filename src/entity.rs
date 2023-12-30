use std::{any::Any, fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::{
    archetype::EntityKey, column::Column, query::fetch::Fetch, Component, Query, QueryShared,
    WorldData,
};

pub trait Columns: Default + 'static {
    type Entity: Entity<Id = EntityKey<Self::Entity>> + EntityVariant<Self::Entity>;

    fn column<C: Component>(&self) -> Option<&Column<C>>;

    fn push(&mut self, entity: Self::Entity);

    fn remove(&mut self, index: usize) -> Self::Entity;

    #[doc(hidden)]
    fn new_fetch<'a>(&self, len: usize) -> <Self::Entity as Entity>::Fetch<'a>;

    #[doc(hidden)]
    fn new_fetch_mut<'a>(&self, len: usize) -> <Self::Entity as Entity>::FetchMut<'a>;
}

pub trait Entity: Sized + 'static {
    type Id: Copy + Debug + Eq + Ord + Hash + 'static;

    type Borrow<'a>: QueryShared + Clone;

    type BorrowMut<'a>: Query;

    type WorldData: WorldData<Entity = Self>;

    #[doc(hidden)]
    type FetchId<'w>: Fetch<Item<'w> = Id<Self>> + 'w;

    #[doc(hidden)]
    type Fetch<'w>: Fetch<Item<'w> = Self::Borrow<'w>> + 'w;

    #[doc(hidden)]
    type FetchMut<'w>: Fetch<Item<'w> = Self::BorrowMut<'w>> + 'w;
}

pub trait CloneEntityFromRef: Entity {
    fn clone_entity_from_ref(entity: Self::Borrow<'_>) -> Self;
}

pub trait CloneEntityIntoRef: EntityStruct {
    fn clone_entity_into_ref(&self, target: &mut Self::BorrowMut<'_>);
}

pub trait EntityStruct: Entity {
    type Columns: Columns<Entity = Self>;
}

pub trait EntityVariant<EOuter: Entity>: Entity {
    fn into_outer(self) -> EOuter;

    fn spawn(self, data: &mut EOuter::WorldData) -> Id<Self>;

    fn id_to_outer(id: Self::Id) -> EOuter::Id
    where
        Self: Sized;

    fn try_id_from_outer(id: EOuter::Id) -> Option<Self::Id>
    where
        Self: Sized;
}

pub type EntityRef<'a, E> = <E as Entity>::Borrow<'a>;

pub type EntityRefMut<'a, E> = <E as Entity>::BorrowMut<'a>;

#[doc(hidden)]
pub type EntityColumns<E> = <E as EntityStruct>::Columns;

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
pub struct Id<E: Entity>(E::Id);

impl<E: Entity> Id<E> {
    pub fn new(id: E::Id) -> Self {
        Self(id)
    }

    pub fn from<EInner: EntityVariant<E>>(id: Id<EInner>) -> Self {
        id.to_outer()
    }

    pub fn get(self) -> E::Id {
        self.0
    }

    pub fn to_outer<EOuter>(self) -> Id<EOuter>
    where
        EOuter: Entity,
        E: EntityVariant<EOuter>,
    {
        Id(E::id_to_outer(self.0))
    }

    pub fn try_to_inner<EInner>(self) -> Option<Id<EInner>>
    where
        EInner: EntityVariant<E>,
    {
        EInner::try_id_from_outer(self.0).map(Id::new)
    }
}

// For proc macros.
#[doc(hidden)]
pub fn downcast_columns_ref<T: Columns, U: Columns>(column: &T) -> Option<&U> {
    (column as &dyn Any).downcast_ref()
}
