use std::{borrow::Borrow, fmt::Debug, hash::Hash, marker::PhantomData};

use frunk::{hlist::Sculptor, prelude::HList};

use crate::Query;

struct WorldBorrow<'a, W, Cs>(&'a mut W, PhantomData<Cs>);

impl<'a, W, Cs> WorldBorrow<'a, W, Cs> {
    pub fn query<Ts, Indices>(
        self,
    ) -> (
        Ts,
        WorldBorrow<'a, W, <Cs as Sculptor<Ts, Indices>>::Remainder>,
    )
    where
        Cs: Sculptor<Ts, Indices>,
    {
        todo!()
    }
}

pub trait World {
    type Id: Copy + Debug + PartialEq + Hash;

    type Entity;

    type Components: HList;

    fn spawn(&mut self, entity: Self::Entity) -> Self::Id;

    fn despawn(&mut self, id: Self::Id) -> Option<Self::Entity>;

    fn query<'a, Q: Query>(&'a mut self) -> Q::Iter<'a>;
}
