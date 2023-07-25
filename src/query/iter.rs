use std::marker::PhantomData;

use crate::{ArchetypeSet, ArchetypeSetFetch};

use super::fetch::Fetch;

// Safety: Before constructing a `FetchIter`, use `BorrowChecker` to ensure that
// the query does not specify borrows that violate Rust's borrowing rules. Also,
// do not allow constructing references to the entity at which the `FetchIter`
// currently points that would violate Rust's borrowing rules.
struct FetchIter<'w, 'f, F, S> {
    i: usize,
    fetch: F,
    _phantom: PhantomData<&'w &'f S>,
}

impl<'w, 'f, F, S> FetchIter<'w, 'f, F, S> {
    pub fn new(fetch: F) -> Self {
        Self {
            i: 0,
            fetch,
            _phantom: PhantomData,
        }
    }
}

impl<'w, 'f, F, S> Iterator for FetchIter<'w, 'f, F, S>
where
    F: Fetch<'w, S>,
    S: ArchetypeSet,
    'w: 'f,
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

pub struct ArchetypeSetFetchIter<'w, 'f, F, S>
where
    F: Fetch<'w, S>,
    S: ArchetypeSet,
    'w: 'f,
{
    archetype_set_iter: <S::Fetch<'w, F> as ArchetypeSetFetch<'w, S>>::Iter,
    current_fetch_iter: Option<FetchIter<'w, 'f, F, S>>,
}

impl<'w, 'f, F, S> Iterator for ArchetypeSetFetchIter<'w, 'f, F, S>
where
    F: Fetch<'w, S>,
    S: ArchetypeSet,
    'w: 'f,
{
    type Item = <F as Fetch<'w, S>>::Item<'f>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item) = self
                .current_fetch_iter
                .as_mut()
                .and_then(|fetch_iter| fetch_iter.next())
            {
                return Some(item);
            }

            self.current_fetch_iter = self.archetype_set_iter.next().map(FetchIter::new);
            if self.current_fetch_iter.is_none() {
                return None;
            }
        }
    }
}

impl<'w, 'f, F, S> ArchetypeSetFetchIter<'w, 'f, F, S>
where
    F: Fetch<'w, S>,
    S: ArchetypeSet,
{
    pub(crate) unsafe fn new(archetype_set: &'w S) -> Self {
        let mut archetype_set_iter = archetype_set.fetch::<F>().iter();

        let current_fetch_iter = archetype_set_iter.next().map(FetchIter::new);

        Self {
            archetype_set_iter,
            current_fetch_iter,
        }
    }
}

pub struct Join<'a, J, S>
where
    J: Fetch<'a, S>,
    S: ArchetypeSet + 'a,
{
    pub(crate) ignore_id: Option<S::EntityId>,
    pub(crate) fetch: S::Fetch<'a, J>,
}

pub struct JoinArchetypeSetFetchIter<'w, F, J, S>
where
    F: Fetch<'w, S>,
    J: Fetch<'w, S>,
    S: ArchetypeSet,
{
    pub(crate) query_iter: ArchetypeSetFetchIter<'w, 'w, F, S>,
    pub(crate) join_fetch: S::Fetch<'w, J>,
}

impl<'w, F, J, S> Iterator for JoinArchetypeSetFetchIter<'w, F, J, S>
where
    F: Fetch<'w, S>,
    J: Fetch<'w, S>,
    S: ArchetypeSet,
{
    type Item = (<F as Fetch<'w, S>>::Item<'w>, Join<'w, J, S>);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.query_iter.next()?;
        let join = Join {
            ignore_id: None,
            fetch: self.join_fetch.clone(),
        };

        Some((item, join))
    }
}

pub struct JoinIter<'a, 'b, J, S, I>
where
    J: Fetch<'a, S>,
    S: ArchetypeSet + 'a,
    'a: 'b,
{
    join: &'b Join<'a, J, S>,
    iter: I,
}

impl<'a, 'b, J, S, I> Iterator for JoinIter<'a, 'b, J, S, I>
where
    J: Fetch<'a, S>,
    S: ArchetypeSet + 'a,
    I: Iterator<Item = S::EntityId>,
    'a: 'b,
{
    type Item = J::Item<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.iter.next()?;

        // Safety: TODO
        unsafe { self.join.fetch.get(id) }
    }
}

impl<'a, J, S> Join<'a, J, S>
where
    J: Fetch<'a, S>,
    S: ArchetypeSet + 'a,
{
    // This has to take an exclusive `self` reference to prevent violating
    // Rust's borrowing rules if `J` contains an exclusive borrow, since `get()`
    // could be called multiple times with the same `id`.
    pub fn get<'b>(&'b mut self, id: S::EntityId) -> Option<J::Item<'b>> {
        if let Some(ignore_id) = self.ignore_id {
            if ignore_id == id {
                // TODO: Consider panicking.
                return None;
            }
        }

        unsafe { self.fetch.get(id) }
    }

    // This has to take an exclusive `self` reference for the same reason as
    // `get()`.
    // FIXME: This does not prevent aliasing.
    pub fn iter<'b, I>(&'b mut self, iter: I) -> JoinIter<'a, 'b, J, S, I>
    where
        'a: 'b,
        I: Iterator<Item = S::EntityId>,
    {
        JoinIter { join: self, iter }
    }
}
