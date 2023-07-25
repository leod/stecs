pub mod fetch;
pub mod iter;

use std::{any::type_name, marker::PhantomData};

use crate::{
    borrow_checker::BorrowChecker,
    column::{ColumnRawParts, ColumnRawPartsMut},
    ArchetypeSet, Component,
};

use self::{
    fetch::{Fetch, FetchEntityId, FetchFromSet, FetchWith, FetchWithout},
    iter::{ArchetypeSetFetchIter, Join, JoinArchetypeSetFetchIter},
};

pub trait Query<S: ArchetypeSet> {
    type Fetch<'w>: FetchFromSet<'w, S>;

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

impl<'q, Q0, Q1, S> Query<S> for (Q0, Q1)
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

pub struct QueryResult<'a, Q, S> {
    archetype_set: &'a mut S,
    _phantom: PhantomData<Q>,
}

impl<'w, Q, S> IntoIterator for QueryResult<'w, Q, S>
where
    Q: Query<S>,
    S: ArchetypeSet,
{
    type Item = <Q::Fetch<'w> as Fetch<'w>>::Item<'w>;

    type IntoIter = ArchetypeSetFetchIter<'w, 'w, Q::Fetch<'w>, S>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        Q::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: A `QueryResult` exclusively borrows the `archetype_set: &'a
        // mut S`. Also, `into_iter` consumes the `QueryResult` while
        // maintaining the lifetime `'a`. Thus, it is not possible to construct
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

    pub fn join_stream<J>(self) -> JoinStreamQueryResult<'w, Q, J, S>
    where
        J: Query<S>,
    {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules. Note that `JoinStreamQueryResult` ensures
        // that `Q` and `J` never borrow the same entity simultaneously, so we
        // can get away with checking their borrows separately. In fact, this
        // separation is the whole purpose of `join_stream`.
        Q::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));
        J::check_borrows(&mut BorrowChecker::new(type_name::<J>()));

        // Safety: TODO
        let query_iter = unsafe { ArchetypeSetFetchIter::new(self.archetype_set) };
        let join_fetch = self.archetype_set.fetch();

        JoinStreamQueryResult {
            query_iter,
            join_fetch,
        }
    }
}

pub struct JoinQueryResult<'a, Q, J, S> {
    archetype_set: &'a mut S,
    _phantom: PhantomData<(Q, J)>,
}

impl<'w, Q, J, S> IntoIterator for JoinQueryResult<'w, Q, J, S>
where
    Q: Query<S>,
    J: Query<S>,
    S: ArchetypeSet,
{
    type Item = (
        <Q::Fetch<'w> as Fetch<'w>>::Item<'w>,
        Join<'w, J::Fetch<'w>, S>,
    );

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

pub struct JoinStreamQueryResult<'w, Q, J, S>
where
    Q: Query<S>,
    J: Query<S>,
    S: ArchetypeSet,
{
    query_iter: ArchetypeSetFetchIter<'w, 'w, (FetchEntityId<S>, Q::Fetch<'w>), S>,
    join_fetch: S::Fetch<'w, J::Fetch<'w>>,
}

impl<'w, Q, J, S> JoinStreamQueryResult<'w, Q, J, S>
where
    Q: Query<S>,
    J: Query<S>,
    S: ArchetypeSet,
{
    pub fn fetch_next(
        &'w mut self,
    ) -> Option<(
        <Q::Fetch<'w> as Fetch<'w>>::Item<'w>,
        Join<'w, J::Fetch<'w>, S>,
    )> {
        let Some((id, item)) = self.query_iter.next() else {
            return None;
        };

        let join = Join {
            ignore_id: Some(id),
            fetch: self.join_fetch.clone(),
        };

        Some((item, join))
    }
}
