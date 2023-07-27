pub mod borrow_checker;
pub mod fetch;
pub mod iter;

use std::{any::type_name, marker::PhantomData};

use crate::{
    column::{ColumnRawParts, ColumnRawPartsMut},
    entity::{Columns, ConcreteEntity},
    Component, EntityRef, EntityRefMut, WorldData,
};

use self::{
    borrow_checker::BorrowChecker,
    fetch::{Fetch, FetchWith, FetchWithout},
    iter::{DataFetchIter, Nest, NestDataFetchIter},
};

pub trait Query<D: WorldData> {
    type Fetch<'w>: Fetch + 'w;
}

impl<'q, C, D> Query<D> for &'q C
where
    C: Component,
    D: WorldData,
{
    type Fetch<'w> = ColumnRawParts<C>;
}

impl<'q, C, D> Query<D> for &'q mut C
where
    C: Component,
    D: WorldData,
{
    type Fetch<'w> = ColumnRawPartsMut<C>;
}

impl<'q, E, D> Query<D> for EntityRef<'q, E>
where
    E: ConcreteEntity,
    D: WorldData,
{
    // FIXME: I'm really not sure if this makes sense at all.
    type Fetch<'f> = <E::Columns as Columns>::Fetch<'f>;
}

impl<'q, E, D> Query<D> for EntityRefMut<'q, E>
where
    E: ConcreteEntity,
    D: WorldData,
{
    // FIXME: I'm really not sure if this makes sense at all.
    type Fetch<'f> = <E::Columns as Columns>::FetchMut<'f>;
}

impl<Q0, Q1, D> Query<D> for (Q0, Q1)
where
    Q0: Query<D>,
    Q1: Query<D>,
    D: WorldData,
{
    type Fetch<'w> = (Q0::Fetch<'w>, Q1::Fetch<'w>);
}

pub struct With<Q, R>(PhantomData<(Q, R)>);

impl<Q, R, D> Query<D> for With<Q, R>
where
    Q: Query<D>,
    R: Query<D>,
    D: WorldData,
{
    type Fetch<'w> = FetchWith<Q::Fetch<'w>, R::Fetch<'w>>;
}

pub struct Without<Q, R>(PhantomData<(Q, R)>);

impl<Q, R, D> Query<D> for Without<Q, R>
where
    Q: Query<D>,
    R: Query<D>,
    D: WorldData,
{
    type Fetch<'w> = FetchWithout<Q::Fetch<'w>, R::Fetch<'w>>;
}

pub struct QueryResult<'w, Q, D> {
    data: &'w mut D,
    _phantom: PhantomData<Q>,
}

impl<'w, Q, D> IntoIterator for QueryResult<'w, Q, D>
where
    Q: Query<D>,
    D: WorldData,
{
    type Item = <Q::Fetch<'w> as Fetch>::Item<'w>;

    type IntoIter = DataFetchIter<'w, Q::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: A `QueryResult` exclusively borrows the `archetype_set: &'w
        // mut S`. Also, `into_iter` consumes the `QueryResult` while
        // maintaining the lifetime `'w`. Thus, it is not possible to construct
        // references to entities in `archetype_set` outside of the returned
        // iterator, thereby satisfying the requirement of `FetchIter`.
        unsafe { DataFetchIter::new(self.data) }
    }
}

impl<'w, Q, D> QueryResult<'w, Q, D>
where
    Q: Query<D>,
    D: WorldData,
{
    pub(crate) fn new(data: &'w mut D) -> Self {
        Self {
            data,
            _phantom: PhantomData,
        }
    }

    pub fn with<R>(self) -> QueryResult<'w, With<Q, R>, D>
    where
        R: Query<D>,
    {
        QueryResult::new(self.data)
    }

    pub fn without<R>(self) -> QueryResult<'w, Without<Q, R>, D>
    where
        R: Query<D>,
    {
        QueryResult::new(self.data)
    }

    pub fn nest<R>(self) -> NestQueryResult<'w, Q, R, D>
    where
        R: Query<D>,
    {
        NestQueryResult {
            archetype_set: self.data,
            _phantom: PhantomData,
        }
    }
}

pub struct NestQueryResult<'w, Q, J, S> {
    archetype_set: &'w mut S,
    _phantom: PhantomData<(Q, J)>,
}

impl<'w, Q, J, D> IntoIterator for NestQueryResult<'w, Q, J, D>
where
    Q: Query<D>,
    J: Query<D>,
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
        let query_iter = unsafe { DataFetchIter::new(self.archetype_set) };
        let nest_fetch = self.archetype_set.fetch();

        NestDataFetchIter {
            query_iter,
            nest_fetch,
        }
    }
}
