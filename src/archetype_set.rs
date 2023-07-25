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

    unsafe fn get<'f>(&self, id: S::AnyEntityId) -> Option<<Self::Fetch as Fetch>::Item<'f>>
    where
        Self: 'f;

    fn iter(&mut self) -> Self::Iter;
}

pub trait ArchetypeSet: Default + Sized {
    type AnyEntityId: Copy + Debug + PartialEq + Query<Self> + 'static;

    type AnyEntity;

    type Fetch<'w, F: FetchFromSet<Self> + 'w>: ArchetypeSetFetch<Self, Fetch = F> + Clone + 'w
    where
        Self: 'w;

    fn spawn<E: InArchetypeSet<Self>>(&mut self, entity: E) -> Self::AnyEntityId;

    fn despawn(&mut self, id: Self::AnyEntityId) -> Option<Self::AnyEntity>;

    #[doc(hidden)]
    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: FetchFromSet<Self> + 'w;

    fn query<Q: Query<Self>>(&mut self) -> QueryResult<Q, Self> {
        QueryResult::new(self)
    }
}

pub type EntityId<S> = <S as ArchetypeSet>::AnyEntityId;

pub trait InArchetypeSet<S: ArchetypeSet>: Entity {
    fn untyped_key_to_key(key: thunderdome::Index) -> EntityKey<Self>;

    fn key_to_id(key: EntityKey<Self>) -> S::AnyEntityId;

    fn into_entity(self) -> S::AnyEntity;
}
