use std::{fmt::Debug, hash::Hash};

use crate::{arena, Archetype, Query};

pub trait World: Default + Sized {
    type EntityId: Copy + Debug + PartialEq + Hash;

    type AnyEntity;

    type QueryIter<'a, Q>: Iterator<Item = Q>
    where
        Self: 'a,
        Q: Query<'a, Self>;

    fn spawn<A: WorldArchetype<Self>>(&mut self, entity: A) -> Self::EntityId;

    fn despawn(&mut self, id: Self::EntityId) -> Option<Self::AnyEntity>;

    fn query<'a, Q>(&'a mut self) -> Self::QueryIter<'a, Q>
    where
        Q: Query<'a, Self>;
}

pub type EntityId<W> = <W as World>::EntityId;

pub type AnyEntity<W> = <W as World>::AnyEntity;

pub trait WorldArchetype<W: World>: Archetype {
    fn id(index: arena::Index) -> W::EntityId;
    fn into_any(self) -> W::AnyEntity;
}
