use std::{fmt::Debug, hash::Hash};

use crate::{Archetype, EntityKey};

pub trait ArchetypeSet: Default + Sized {
    type EntityId: Copy + Debug + PartialEq + Hash;

    type Entity;

    /*type QueryIter<'a, Q>: Iterator<Item = Q>
    where
        Self: 'a,
        Q: Query<'a, Self>;

    fn spawn<A: ArchetypeInSet<Self>>(&mut self, entity: A) -> Self::EntityId;

    fn despawn(&mut self, id: Self::EntityId) -> Option<Self::Entity>;

    fn query<'a, Q>(&'a mut self) -> Self::QueryIter<'a, Q>
    where
        Q: Query<'a, Self>;*/
}

pub type EntityId<W> = <W as ArchetypeSet>::EntityId;

pub type Entity<W> = <W as ArchetypeSet>::Entity;

pub trait ArchetypeInSet<S: ArchetypeSet>: Archetype {
    fn id(key: EntityKey<Self>) -> S::EntityId;

    fn into_entity(self) -> S::Entity;
}
