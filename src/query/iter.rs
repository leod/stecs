use std::marker::PhantomData;

use crate::{world::WorldFetch, Entity, WorldData};

use super::fetch::Fetch;

// Safety: Before constructing a `FetchIter`, use `BorrowChecker` to ensure that
// the query does not specify borrows that violate Rust's borrowing rules. Also,
// do not allow constructing references to the entity at which the `FetchIter`
// currently points that would violate Rust's borrowing rules.
pub(crate) struct FetchIter<'f, F> {
    i: usize,
    fetch: F,
    _phantom: PhantomData<&'f ()>,
}

impl<'f, F> FetchIter<'f, F> {
    pub fn new(fetch: F) -> Self {
        Self {
            i: 0,
            fetch,
            _phantom: PhantomData,
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

pub struct DataFetchIter<'w, F, D>
where
    F: Fetch + 'w,
    D: WorldData + 'w,
{
    data_iter: <D::Fetch<'w, F> as WorldFetch<'w, D>>::Iter,
    current_fetch_iter: Option<FetchIter<'w, F>>,
}

impl<'w, F, D> Iterator for DataFetchIter<'w, F, D>
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

            self.current_fetch_iter = self.data_iter.next().map(FetchIter::new);
            self.current_fetch_iter.as_ref()?;
        }
    }
}

impl<'w, F, D> DataFetchIter<'w, F, D>
where
    F: Fetch,
    D: WorldData,
{
    pub(crate) unsafe fn new(data: &'w D) -> Self {
        let mut data_iter = data.fetch::<F>().iter();
        let current_fetch_iter = data_iter.next().map(FetchIter::new);

        Self {
            data_iter,
            current_fetch_iter,
        }
    }
}

pub struct Nest<'w, J, D>
where
    J: Fetch + 'w,
    D: WorldData + 'w,
{
    pub(crate) ignore_id: Option<<D::Entity as Entity>::Id>,
    pub(crate) fetch: D::Fetch<'w, J>,
}

pub struct NestDataFetchIter<'w, F, J, D>
where
    F: Fetch,
    J: Fetch + 'w,
    D: WorldData,
{
    pub(crate) query_iter: DataFetchIter<'w, F, D>,
    pub(crate) nest_fetch: D::Fetch<'w, J>,
}

impl<'w, F, J, D> Iterator for NestDataFetchIter<'w, F, J, D>
where
    F: Fetch + 'w,
    J: Fetch + 'w,
    D: WorldData,
{
    type Item = (<F as Fetch>::Item<'w>, Nest<'w, J, D>);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.query_iter.next()?;
        let nest = Nest {
            ignore_id: None,
            fetch: self.nest_fetch.clone(),
        };

        Some((item, nest))
    }
}

impl<'w, J, D> Nest<'w, J, D>
where
    J: Fetch,
    D: WorldData + 'w,
{
    // This has to take an exclusive `self` reference to prevent violating
    // Rust's borrowing rules if `J` contains an exclusive borrow, since `get()`
    // could be called multiple times with the same `id`.
    pub fn get<'f>(&'w mut self, id: <D::Entity as Entity>::Id) -> Option<J::Item<'f>>
    where
        'w: 'f,
    {
        if let Some(ignore_id) = self.ignore_id {
            if ignore_id == id {
                // TODO: Consider panicking.
                return None;
            }
        }

        unsafe { self.fetch.get(id) }
    }
}
