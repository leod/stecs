use std::{fmt::Debug, hash::Hash};

use crate::Query;

pub trait World {
    type Id: Copy + Debug + PartialEq + Hash;

    type Entity;

    fn spawn(&mut self, entity: Self::Entity) -> Self::Id;

    fn despawn(&mut self, id: Self::Id) -> Option<Self::Entity>;

    fn query<'a, Q: Query>(&'a mut self) -> Q::Iter<'a>;
}
