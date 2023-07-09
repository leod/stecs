use std::{fmt::Debug, hash::Hash};

use crate::Query;

pub trait World: Default + Sized {
    type EntityId: Copy + Debug + PartialEq + Hash;

    type Entity;

    type QueryIter<'a, Q>: Iterator<Item = Q>
    where
        Self: 'a,
        Q: Query<'a>;

    fn spawn(&mut self, entity: impl Into<Self::Entity>) -> Self::EntityId;

    fn despawn(&mut self, id: Self::EntityId) -> Option<Self::Entity>;

    fn query<'a, Q>(&'a mut self) -> Self::QueryIter<'a, Q>
    where
        Q: Query<'a>;
}

pub type EntityId<W> = <W as World>::EntityId;

pub type Entity<W> = <W as World>::Entity;
