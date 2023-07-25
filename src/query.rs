pub mod fetch;
pub mod iter;

use std::{any::type_name, marker::PhantomData};

use crate::{
    borrow_checker::BorrowChecker,
    column::{ColumnRawParts, ColumnRawPartsMut},
    entity::EntityBorrow,
    ArchetypeSet, Component, Entity, EntityRef, EntityRefMut,
};

use self::{
    fetch::{Fetch, FetchFromSet, FetchWith, FetchWithout},
    iter::{ArchetypeSetFetchIter, Join, JoinArchetypeSetFetchIter},
};

pub trait Query<S: ArchetypeSet> {
    type Fetch<'w>: FetchFromSet<S> + 'w;

    #[doc(hidden)]
    fn check_borrows(checker: &mut BorrowChecker);
}

impl<'q, C, S> Query<S> for &'q C
where
    C: Component,
    S: ArchetypeSet,
{
    type Fetch<'w> = ColumnRawParts<C>;

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow::<C>();
    }
}

impl<'q, C, S> Query<S> for &'q mut C
where
    C: Component,
    S: ArchetypeSet,
{
    type Fetch<'w> = ColumnRawPartsMut<C>;

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow_mut::<C>();
    }
}

impl<'q, E, S> Query<S> for EntityRef<'q, E>
where
    E: Entity,
    for<'w> <E::Borrow<'w> as EntityBorrow<'w>>::Fetch<'w>: FetchFromSet<S>,
    S: ArchetypeSet,
{
    // FIXME: I'm really not sure if this makes sense at all.
    type Fetch<'w> = <E::Borrow<'w> as EntityBorrow<'w>>::Fetch<'w>;

    fn check_borrows(checker: &mut BorrowChecker) {
        // TODO -> move to Fetch
    }
}

impl<'q, E, S> Query<S> for EntityRefMut<'q, E>
where
    E: Entity,
    for<'w> <E::BorrowMut<'w> as EntityBorrow<'w>>::Fetch<'w>: FetchFromSet<S>,
    S: ArchetypeSet,
{
    // FIXME: I'm really not sure if this makes sense at all.
    type Fetch<'w> = <E::BorrowMut<'w> as EntityBorrow<'w>>::Fetch<'w>;

    fn check_borrows(checker: &mut BorrowChecker) {
        // TODO -> move to Fetch
    }
}

impl<Q0, Q1, S> Query<S> for (Q0, Q1)
where
    Q0: Query<S>,
    Q1: Query<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = (Q0::Fetch<'w>, Q1::Fetch<'w>);

    fn check_borrows(checker: &mut BorrowChecker) {
        Q0::check_borrows(checker);
        Q1::check_borrows(checker);
    }
}

pub struct With<Q, R>(PhantomData<(Q, R)>);

impl<Q, R, S> Query<S> for With<Q, R>
where
    Q: Query<S>,
    R: Query<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = FetchWith<Q::Fetch<'w>, R::Fetch<'w>>;

    fn check_borrows(checker: &mut BorrowChecker) {
        Q::check_borrows(checker);
    }
}

pub struct Without<Q, R>(PhantomData<(Q, R)>);

impl<Q, R, S> Query<S> for Without<Q, R>
where
    Q: Query<S>,
    R: Query<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = FetchWithout<Q::Fetch<'w>, R::Fetch<'w>>;

    fn check_borrows(checker: &mut BorrowChecker) {
        Q::check_borrows(checker);
    }
}

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
        Q::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

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

    pub fn join<J>(self) -> JoinQueryResult<'w, Q, J, S>
    where
        J: Query<S>,
    {
        JoinQueryResult {
            archetype_set: self.archetype_set,
            _phantom: PhantomData,
        }
    }
}

pub struct JoinQueryResult<'w, Q, J, S> {
    archetype_set: &'w mut S,
    _phantom: PhantomData<(Q, J)>,
}

impl<'w, Q, J, S> IntoIterator for JoinQueryResult<'w, Q, J, S>
where
    Q: Query<S>,
    J: Query<S>,
    S: ArchetypeSet,
{
    type Item = (<Q::Fetch<'w> as Fetch>::Item<'w>, Join<'w, J::Fetch<'w>, S>);

    type IntoIter = JoinArchetypeSetFetchIter<'w, Q::Fetch<'w>, J::Fetch<'w>, S>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        Q::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: TODO
        let query_iter = unsafe { ArchetypeSetFetchIter::new(self.archetype_set) };
        let join_fetch = self.archetype_set.fetch();

        JoinArchetypeSetFetchIter {
            query_iter,
            join_fetch,
        }
    }
}
