pub mod borrow_checker;
pub mod fetch;
pub mod iter;

use std::{any::type_name, marker::PhantomData};

use crate::{
    column::{ColumnRawParts, ColumnRawPartsMut},
    entity::EntityVariant,
    world::WorldFetch,
    Component, Entity, EntityId, WorldData,
};

use self::{
    borrow_checker::BorrowChecker,
    fetch::{Fetch, UnitFetch, WithFetch, WithoutFetch},
    iter::{DataFetchIter, Nest, NestDataFetchIter},
};

pub trait Query {
    // TODO: Strongly consider getting rid of the 'w lifetime. It makes traits
    // much more complex. Also, `Fetch` is not directly used by users, and its
    // main method `get` already is `unsafe`, i.e. we can ensure within the
    // library that the borrowed world data still lives.
    type Fetch<'w>: Fetch + 'w;
}

pub trait QueryShared: Query {}

impl<'q, C: Component> Query for &'q C {
    type Fetch<'w> = ColumnRawParts<C>;
}

impl<'q, C: Component> QueryShared for &'q C {}

impl<'q, C: Component> Query for &'q mut C {
    type Fetch<'w> = ColumnRawPartsMut<C>;
}

impl<E: Entity> Query for EntityId<E> {
    type Fetch<'w> = E::FetchId<'w>;
}

impl<E: Entity> QueryShared for EntityId<E> {}

macro_rules! tuple_impl {
    () => {
        impl Query for () {
            type Fetch<'w> = UnitFetch;
        }
    };
    ($($name: ident),*) => {
        impl<$($name: Query,)*> Query for ($($name,)*) {
            type Fetch<'w> = ($($name::Fetch<'w>,)*);
        }

        impl<$($name: QueryShared,)*> QueryShared for ($($name,)*) {
        }
    };
}

smaller_tuples_too!(
    tuple_impl, F0, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15
);

pub struct With<Q, R>(PhantomData<(Q, R)>);

impl<Q, R> Query for With<Q, R>
where
    Q: Query,
    R: Query,
{
    type Fetch<'w> = WithFetch<Q::Fetch<'w>, R::Fetch<'w>>;
}

impl<Q, R> QueryShared for With<Q, R>
where
    Q: QueryShared,
    R: Query,
{
}

pub struct Without<Q, R>(PhantomData<(Q, R)>);

impl<Q, R> Query for Without<Q, R>
where
    Q: Query,
    R: Query,
{
    type Fetch<'w> = WithoutFetch<Q::Fetch<'w>, R::Fetch<'w>>;
}

impl<Q, R> QueryShared for Without<Q, R>
where
    Q: QueryShared,
    R: Query,
{
}

pub struct QueryBorrow<'w, Q, D> {
    data: &'w D,
    _phantom: PhantomData<Q>,
}

impl<'w, Q, D> IntoIterator for QueryBorrow<'w, Q, D>
where
    Q: Query,
    D: WorldData,
{
    type Item = <Q::Fetch<'w> as Fetch>::Item<'w>;

    type IntoIter = DataFetchIter<'w, Q::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: A `QueryResult` exclusively borrows the `data: &'w mut D`.
        // Also, `into_iter` consumes the `QueryResult` while maintaining the
        // lifetime `'w`. Thus, it is not possible to construct references to
        // entities in `data` outside of the returned iterator, thereby
        // satisfying the requirement of `FetchIter`.
        unsafe { DataFetchIter::new(self.data) }
    }
}

impl<'w, Q, D> QueryBorrow<'w, Q, D>
where
    Q: Query,
    D: WorldData,
{
    pub(crate) fn new(data: &'w D) -> Self {
        Self {
            data,
            _phantom: PhantomData,
        }
    }

    pub fn with<R>(self) -> QueryBorrow<'w, With<Q, R>, D>
    where
        R: Query,
    {
        QueryBorrow::new(self.data)
    }

    pub fn without<R>(self) -> QueryBorrow<'w, Without<Q, R>, D>
    where
        R: Query,
    {
        QueryBorrow::new(self.data)
    }

    pub fn nest<R>(self) -> NestQueryBorrow<'w, Q, R, D>
    where
        R: Query,
    {
        NestQueryBorrow {
            data: self.data,
            _phantom: PhantomData,
        }
    }

    pub fn get<'f, E>(&'f self, id: EntityId<E>) -> Option<<Q::Fetch<'f> as Fetch>::Item<'f>>
    where
        'w: 'f,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        // TODO: Cache?

        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'f> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        let world_fetch = self.data.fetch::<Q::Fetch<'f>>();

        // Safety: TODO
        unsafe { world_fetch.get(id.get()) }
    }

    pub fn get_mut<'f, E>(
        &'f mut self,
        id: EntityId<E>,
    ) -> Option<<Q::Fetch<'f> as Fetch>::Item<'f>>
    where
        'w: 'f,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        // TODO: Cache?

        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'f> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        let world_fetch = self.data.fetch::<Q::Fetch<'f>>();

        // Safety: TODO
        unsafe { world_fetch.get(id.get()) }
    }
}

pub struct NestQueryBorrow<'w, Q, J, S> {
    data: &'w S,
    _phantom: PhantomData<(Q, J)>,
}

// TODO: Implement `get` for `NestQueryResult`.

// TODO: Implement `nest` for `NestQueryResult`; require non-overlapping borrows
// for multiple nests.

impl<'w, Q, J, D> IntoIterator for NestQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: Query,
    D: WorldData,
{
    type Item = (<Q::Fetch<'w> as Fetch>::Item<'w>, Nest<'w, J::Fetch<'w>, D>);

    type IntoIter = NestDataFetchIter<'w, Q::Fetch<'w>, J::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));
        <J::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<J>()));

        // Safety: TODO
        let query_iter = unsafe { DataFetchIter::new(self.data) };
        let nest_fetch = self.data.fetch();

        NestDataFetchIter {
            query_iter,
            nest_fetch,
        }
    }
}
