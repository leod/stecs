use std::{fmt::Debug, hash::Hash};

use crate::{Entity, EntityKey};

pub trait ArchetypeSet: Default + Sized {
    type EntityId: Copy;

    type Entity;

    fn spawn<E: InArchetypeSet<Self>>(&mut self, entity: E) -> Self::EntityId;

    fn despawn(&mut self, id: Self::EntityId) -> Option<Self::Entity>;

    /*
    type QueryIter<'a, Q>: Iterator<Item = Q>;
    where
        Self: 'a,
        Q: Query<'a, Self>;

    fn query<'a, Q>(&'a mut self) -> Self::QueryIter<'a, Q>
    where
        Q: Query<'a, Self>;
    */

    type QueryIter<Q>: Iterator<Item = Q>;

    fn query<'a, Q>(&'a mut self) -> Self::QueryIter<Q>;
}

pub type EntityId<S> = <S as ArchetypeSet>::EntityId;

pub trait InArchetypeSet<S: ArchetypeSet>: Entity {
    fn id(key: EntityKey<Self>) -> S::EntityId;

    fn into_entity(self) -> S::Entity;
}
