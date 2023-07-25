use std::marker::PhantomData;

use crate::{archetype_set::ArchetypeSetFetch, ArchetypeSet};

use super::fetch::{Fetch, FetchFromSet};

// Safety: Before constructing a `FetchIter`, use `BorrowChecker` to ensure that
// the query does not specify borrows that violate Rust's borrowing rules. Also,
// do not allow constructing references to the entity at which the `FetchIter`
// currently points that would violate Rust's borrowing rules.
struct FetchIter<'f, F, S> {
    i: usize,
    fetch: F,
    _phantom: PhantomData<&'f S>,
}

impl<'f, F, S> FetchIter<'f, F, S> {
    pub fn new(fetch: F) -> Self {
        Self {
            i: 0,
            fetch,
            _phantom: PhantomData,
        }
    }
}

impl<'f, F, S> Iterator for FetchIter<'f, F, S>
where
    F: FetchFromSet<S> + 'f,
    S: ArchetypeSet,
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

pub struct ArchetypeSetFetchIter<'w, F, S>
where
    F: FetchFromSet<S>,
    S: ArchetypeSet + 'w,
{
    archetype_set_iter: <S::Fetch<'w, F> as ArchetypeSetFetch<S>>::Iter,
    current_fetch_iter: Option<FetchIter<'w, F, S>>,
}

impl<'w, F, S> Iterator for ArchetypeSetFetchIter<'w, F, S>
where
    F: FetchFromSet<S> + 'w,
    S: ArchetypeSet,
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

            self.current_fetch_iter = self.archetype_set_iter.next().map(FetchIter::new);
            self.current_fetch_iter.as_ref()?;
        }
    }
}

impl<'w, F, S> ArchetypeSetFetchIter<'w, F, S>
where
    F: FetchFromSet<S>,
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

pub struct Join<'w, J, S>
where
    J: FetchFromSet<S>,
    S: ArchetypeSet + 'w,
{
    pub(crate) ignore_id: Option<S::EntityId>,
    pub(crate) fetch: S::Fetch<'w, J>,
}

pub struct JoinArchetypeSetFetchIter<'w, F, J, S>
where
    F: FetchFromSet<S>,
    J: FetchFromSet<S>,
    S: ArchetypeSet,
{
    pub(crate) query_iter: ArchetypeSetFetchIter<'w, F, S>,
    pub(crate) join_fetch: S::Fetch<'w, J>,
}

impl<'w, F, J, S> Iterator for JoinArchetypeSetFetchIter<'w, F, J, S>
where
    F: FetchFromSet<S> + 'w,
    J: FetchFromSet<S>,
    S: ArchetypeSet,
{
    type Item = (<F as Fetch>::Item<'w>, Join<'w, J, S>);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.query_iter.next()?;
        let join = Join {
            ignore_id: None,
            fetch: self.join_fetch.clone(),
        };

        Some((item, join))
    }
}

impl<'a, J, S> Join<'a, J, S>
where
    J: FetchFromSet<S>,
    S: ArchetypeSet + 'a,
{
    // This has to take an exclusive `self` reference to prevent violating
    // Rust's borrowing rules if `J` contains an exclusive borrow, since `get()`
    // could be called multiple times with the same `id`.
    pub fn get(&mut self, id: S::EntityId) -> Option<J::Item<'_>> {
        if let Some(ignore_id) = self.ignore_id {
            if ignore_id == id {
                // TODO: Consider panicking.
                return None;
            }
        }

        unsafe { self.fetch.get(id) }
    }
}
