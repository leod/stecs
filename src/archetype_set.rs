use std::fmt::Debug;

use crate::{
    query::{Fetch, QueryResult},
    Entity, EntityKey, Query,
};

pub trait ArchetypeSetFetch<'a, S: ArchetypeSet> {
    type Fetch: Fetch<'a, S>;
    type Iter: Iterator<Item = Self::Fetch>;

    unsafe fn get(&self, id: S::EntityId) -> Option<<Self::Fetch as Fetch<'a, S>>::Item>;

    fn iter(&mut self) -> Self::Iter;
}

pub trait ArchetypeSet: Default + Sized {
    type EntityId: Copy + Debug + for<'a> Query<'a, Self> + 'static;

    type Entity;

    type Fetch<'a, F: Fetch<'a, Self>>: ArchetypeSetFetch<'a, Self, Fetch = F>
    where
        Self: 'a;

    fn spawn<E: InArchetypeSet<Self>>(&mut self, entity: E) -> Self::EntityId;

    fn despawn(&mut self, id: Self::EntityId) -> Option<Self::Entity>;

    fn fetch<'a, F>(&'a mut self) -> Self::Fetch<'a, F>
    where
        F: Fetch<'a, Self>;

    fn query<'a, Q>(&'a mut self) -> QueryResult<'a, Q, Self>
    where
        Q: Query<'a, Self>,
    {
        QueryResult::new(self)
    }
}

pub type EntityId<S> = <S as ArchetypeSet>::EntityId;

pub trait InArchetypeSet<S: ArchetypeSet>: Entity {
    fn untyped_key_to_key(key: thunderdome::Index) -> EntityKey<Self>;

    fn key_to_id(key: EntityKey<Self>) -> S::EntityId;

    fn into_entity(self) -> S::Entity;
}
