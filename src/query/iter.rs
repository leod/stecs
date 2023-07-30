use std::{any::type_name, marker::PhantomData};

use crate::{world::WorldFetch, QueryShared, WorldData};

use super::{borrow_checker::BorrowChecker, fetch::Fetch, Query, QueryBorrow};

impl<'w, Q, D> IntoIterator for QueryBorrow<'w, Q, D>
where
    Q: Query,
    D: WorldData,
{
    type Item = <Q::Fetch<'w> as Fetch>::Item<'w>;

    type IntoIter = WorldFetchIter<'w, Q::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: A `QueryResult` exclusively borrows the `data: &'w mut D`.
        // Also, `into_iter` consumes the `QueryResult` while maintaining the
        // lifetime `'w`. Thus, it is not possible to construct references to
        // entities in `data` outside of the returned iterator, thereby
        // satisfying the requirement of `FetchIter`.
        unsafe { WorldFetchIter::new(self.data) }
    }
}

impl<'w, Q, D> IntoIterator for &'w QueryBorrow<'w, Q, D>
where
    Q: QueryShared,
    D: WorldData,
{
    type Item = <Q::Fetch<'w> as Fetch>::Item<'w>;

    type IntoIter = WorldFetchIter<'w, Q::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: A `QueryResult` exclusively borrows the `data: &'w mut D`.
        // Also, `into_iter` consumes the `QueryResult` while maintaining the
        // lifetime `'w`. Thus, it is not possible to construct references to
        // entities in `data` outside of the returned iterator, thereby
        // satisfying the requirement of `FetchIter`.
        unsafe { WorldFetchIter::new(self.data) }
    }
}

// Safety: Before constructing a `FetchIter`, use `BorrowChecker` to ensure that
// the query does not specify borrows that violate Rust's borrowing rules. Also,
// do not allow constructing references to the entity at which the `FetchIter`
// currently points that would violate Rust's borrowing rules.
pub(crate) struct FetchIter<'f, F> {
    i: usize,
    fetch: F,
    _phantom: PhantomData<&'f ()>,
}

impl<'f, F> FetchIter<'f, F>
where
    F: Fetch + 'f,
{
    pub fn new(fetch: F) -> Self {
        Self {
            i: 0,
            fetch,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn skip_one(&mut self) {
        if self.i < self.fetch.len() {
            self.i += 1;
        }
    }
}

impl<'f, F> Iterator for FetchIter<'f, F>
where
    F: Fetch + 'f,
{
    type Item = F::Item<'f>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.fetch.len() {
            None
        } else {
            // Safety: See the comment on `FetchIter`.
            let item = unsafe { self.fetch.get(self.i) };

            self.i += 1;

            Some(item)
        }
    }
}

pub struct WorldFetchIter<'w, F, D>
where
    F: Fetch + 'w,
    D: WorldData + 'w,
{
    len: usize,
    world_iter: <D::Fetch<'w, F> as WorldFetch<'w, D>>::Iter,
    current_fetch_iter: Option<FetchIter<'w, F>>,
}

impl<'w, F, D> Iterator for WorldFetchIter<'w, F, D>
where
    F: Fetch + 'w,
    D: WorldData,
{
    type Item = <F as Fetch>::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item) = self
                .current_fetch_iter
                .as_mut()
                .and_then(|fetch_iter| fetch_iter.next())
            {
                return Some(item);
            }

            self.current_fetch_iter = self.world_iter.next().map(FetchIter::new);
            self.current_fetch_iter.as_ref()?;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'w, F, D> ExactSizeIterator for WorldFetchIter<'w, F, D>
where
    F: Fetch + 'w,
    D: WorldData,
{
}

impl<'w, F, D> WorldFetchIter<'w, F, D>
where
    F: Fetch,
    D: WorldData,
{
    pub(crate) unsafe fn new(data: &'w D) -> Self {
        let mut world_fetch = data.fetch::<F>();
        let len = world_fetch.len();
        let mut world_iter = world_fetch.iter();
        let current_fetch_iter = world_iter.next().map(FetchIter::new);

        Self {
            len,
            world_iter,
            current_fetch_iter,
        }
    }

    pub(crate) fn skip_one(&mut self) {
        if let Some(fetch_iter) = self.current_fetch_iter.as_mut() {
            fetch_iter.skip_one();
        }
    }
}
