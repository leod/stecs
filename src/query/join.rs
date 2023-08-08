use std::marker::PhantomData;

use crate::{
    secondary::query::SecondaryFetch, Entity, Query, SecondaryQuery, SecondaryWorld, WorldData,
};

use super::{fetch::Fetch, iter::WorldFetchIter, QueryItem};

pub struct JoinQueryBorrow<'w, Q, J, D>
where
    D: WorldData,
{
    pub(crate) data: &'w D,
    pub(crate) secondary_world: &'w SecondaryWorld<D::Entity>,
    pub(crate) _phantom: PhantomData<(Q, J)>,
}

// TODO: `get` and `get_mut` for JoinQueryBorrow

impl<'w, Q, J, D> IntoIterator for JoinQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: SecondaryQuery<D::Entity>,
    D: WorldData,
{
    type Item = (
        QueryItem<'w, Q>,
        <J::Fetch<'w> as SecondaryFetch<'w, D::Entity>>::Item<'w>,
    );

    type IntoIter = JoinQueryFetchIter<'w, Q::Fetch<'w>, J::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        super::assert_borrow::<Q>();
        crate::secondary::query::assert_borrow::<D::Entity, J>();

        // Safety: TODO
        let query_iter = unsafe { WorldFetchIter::new(self.data) };

        let secondary_fetch =
            <J::Fetch<'w> as SecondaryFetch<'w, D::Entity>>::new(self.secondary_world);

        JoinQueryFetchIter {
            query_iter,
            secondary_fetch,
        }
    }
}

pub struct JoinQueryFetchIter<'w, F, J, D>
where
    F: Fetch + 'w,
    J: SecondaryFetch<'w, D::Entity>,
    D: WorldData,
{
    query_iter: WorldFetchIter<'w, (<D::Entity as Entity>::FetchId<'w>, F), D>,
    secondary_fetch: Option<J>,
}

impl<'w, F, J, D> Iterator for JoinQueryFetchIter<'w, F, J, D>
where
    F: Fetch + 'w,
    J: SecondaryFetch<'w, D::Entity>,
    D: WorldData,
{
    type Item = (F::Item<'w>, J::Item<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, item) = self.query_iter.next()?;
        let secondary_fetch = self.secondary_fetch.as_ref()?;

        // Safety: `FetchIter` does not generate duplicate IDs.
        unsafe { secondary_fetch.get(id) }.map(|secondary_item| (item, secondary_item))
    }
}
