pub mod borrow_checker;
pub mod fetch;
pub mod iter;

use std::{any::type_name, marker::PhantomData};

use crate::{
    archetype_set::InArchetypeSet,
    column::{ColumnRawParts, ColumnRawPartsMut},
    entity::EntityBorrow,
    ArchetypeSet, Component, Entity, EntityId, EntityRef, EntityRefMut,
};

use self::{
    borrow_checker::BorrowChecker,
    fetch::{Fetch, FetchWith, FetchWithout},
    iter::{ArchetypeSetFetchIter, Nest, NestArchetypeSetFetchIter},
};

pub trait Query<S: ArchetypeSet> {
    type Fetch<'w>: Fetch + 'w;
}

impl<'q, C, S> Query<S> for &'q C
where
    C: Component,
    S: ArchetypeSet,
{
    type Fetch<'w> = ColumnRawParts<C>;
}

impl<'q, C, S> Query<S> for &'q mut C
where
    C: Component,
    S: ArchetypeSet,
{
    type Fetch<'w> = ColumnRawPartsMut<C>;
}

impl<'q, E, S> Query<S> for EntityRef<'q, E>
where
    E: Entity,
    for<'w> <E::Borrow<'w> as EntityBorrow<'w>>::Fetch<'w>: Fetch,
    S: ArchetypeSet,
{
    // FIXME: I'm really not sure if this makes sense at all.
    type Fetch<'w> = <E::Borrow<'w> as EntityBorrow<'w>>::Fetch<'w>;
}

impl<'q, E, S> Query<S> for EntityRefMut<'q, E>
where
    E: Entity,
    for<'w> <E::BorrowMut<'w> as EntityBorrow<'w>>::Fetch<'w>: Fetch,
    S: ArchetypeSet,
{
    // FIXME: I'm really not sure if this makes sense at all.
    type Fetch<'w> = <E::BorrowMut<'w> as EntityBorrow<'w>>::Fetch<'w>;
}

impl<Q0, Q1, S> Query<S> for (Q0, Q1)
where
    Q0: Query<S>,
    Q1: Query<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = (Q0::Fetch<'w>, Q1::Fetch<'w>);
}

pub struct With<Q, R>(PhantomData<(Q, R)>);

impl<Q, R, S> Query<S> for With<Q, R>
where
    Q: Query<S>,
    R: Query<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = FetchWith<Q::Fetch<'w>, R::Fetch<'w>>;
}

pub struct Without<Q, R>(PhantomData<(Q, R)>);

impl<Q, R, S> Query<S> for Without<Q, R>
where
    Q: Query<S>,
    R: Query<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = FetchWithout<Q::Fetch<'w>, R::Fetch<'w>>;
}

/*
impl<E, S> Query<S> for EntityId<E>
where
    E: InArchetypeSet<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = FetchEntityId<E>;
}
*/

pub struct QueryResult<'w, Q, S> {
    archetype_set: &'w mut S,
    _phantom: PhantomData<Q>,
}

impl<'w, Q, S> IntoIterator for QueryResult<'w, Q, S>
where
    Q: Query<S>,
    S: ArchetypeSet,
{
    type Item = <Q::Fetch<'w> as Fetch>::Item<'w>;

    type IntoIter = ArchetypeSetFetchIter<'w, Q::Fetch<'w>, S>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: A `QueryResult` exclusively borrows the `archetype_set: &'w
        // mut S`. Also, `into_iter` consumes the `QueryResult` while
        // maintaining the lifetime `'w`. Thus, it is not possible to construct
        // references to entities in `archetype_set` outside of the returned
        // iterator, thereby satisfying the requirement of `FetchIter`.
        unsafe { ArchetypeSetFetchIter::new(self.archetype_set) }
    }
}

impl<'w, Q, S> QueryResult<'w, Q, S>
where
    Q: Query<S>,
    S: ArchetypeSet,
{
    pub(crate) fn new(archetype_set: &'w mut S) -> Self {
        Self {
            archetype_set,
            _phantom: PhantomData,
        }
    }

    pub fn with<R>(self) -> QueryResult<'w, With<Q, R>, S>
    where
        R: Query<S>,
    {
        QueryResult::new(self.archetype_set)
    }

    pub fn without<R>(self) -> QueryResult<'w, Without<Q, R>, S>
    where
        R: Query<S>,
    {
        QueryResult::new(self.archetype_set)
    }

    pub fn nest<R>(self) -> NestQueryResult<'w, Q, R, S>
    where
        R: Query<S>,
    {
        NestQueryResult {
            archetype_set: self.archetype_set,
            _phantom: PhantomData,
        }
    }
}

pub struct NestQueryResult<'w, Q, J, S> {
    archetype_set: &'w mut S,
    _phantom: PhantomData<(Q, J)>,
}

impl<'w, Q, J, S> IntoIterator for NestQueryResult<'w, Q, J, S>
where
    Q: Query<S>,
    J: Query<S>,
    S: ArchetypeSet,
{
    type Item = (<Q::Fetch<'w> as Fetch>::Item<'w>, Nest<'w, J::Fetch<'w>, S>);

    type IntoIter = NestArchetypeSetFetchIter<'w, Q::Fetch<'w>, J::Fetch<'w>, S>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: TODO
        let query_iter = unsafe { ArchetypeSetFetchIter::new(self.archetype_set) };
        let nest_fetch = self.archetype_set.fetch();

        NestArchetypeSetFetchIter {
            query_iter,
            nest_fetch,
        }
    }
}
