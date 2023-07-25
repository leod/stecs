use std::fmt::Debug;

use crate::{
    query::{
        fetch::{Fetch, FetchFromSet},
        QueryResult,
    },
    Entity, EntityKey, Query,
};

pub trait ArchetypeSetFetch<S: ArchetypeSet> {
    type Fetch: FetchFromSet<S>;
    type Iter: Iterator<Item = Self::Fetch>;

    unsafe fn get<'b>(&self, id: S::EntityId) -> Option<<Self::Fetch as Fetch>::Item<'b>>;

    fn iter(&mut self) -> Self::Iter;
}

pub trait ArchetypeSet: Default + Sized {
    type EntityId: Copy + Debug + PartialEq + Query<Self> + 'static;

    type Entity;

    type Fetch<'w, F: FetchFromSet<Self>>: ArchetypeSetFetch<Self, Fetch = F> + Clone
    where
        Self: 'w;

    fn spawn<E: InArchetypeSet<Self>>(&mut self, entity: E) -> Self::EntityId;

    fn despawn(&mut self, id: Self::EntityId) -> Option<Self::Entity>;

    #[doc(hidden)]
    fn fetch<F>(&self) -> Self::Fetch<'_, F>
    where
        F: FetchFromSet<Self>;

    fn query<Q: Query<Self>>(&mut self) -> QueryResult<Q, Self> {
        QueryResult::new(self)
    }
}

pub type EntityId<S> = <S as ArchetypeSet>::EntityId;

pub trait InArchetypeSet<S: ArchetypeSet>: Entity {
    fn untyped_key_to_key(key: thunderdome::Index) -> EntityKey<Self>;

    fn key_to_id(key: EntityKey<Self>) -> S::EntityId;

    fn into_entity(self) -> S::Entity;
}
