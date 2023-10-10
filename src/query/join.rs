use std::marker::PhantomData;

use crate::{
    entity::EntityVariant,
    secondary::query::{SecondaryFetch, SecondaryQueryItem},
    world::WorldFetch,
    Entity, EntityId, Query, QueryShared, SecondaryQuery, SecondaryQueryShared, SecondaryWorld,
    WorldData,
};

use super::{fetch::Fetch, iter::WorldFetchIter, QueryItem};

pub struct JoinQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: SecondaryQuery<D::Entity>,
    D: WorldData,
{
    pub(crate) data: &'w D,
    pub(crate) fetch: D::Fetch<'w, Q::Fetch<'w>>,
    pub(crate) secondary_fetch: Option<J::Fetch<'w>>,
    pub(crate) _phantom: PhantomData<(Q, J)>,
}

impl<'w, Q, J, D> JoinQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: SecondaryQuery<D::Entity>,
    D: WorldData,
{
    pub fn new(data: &'w D, secondary_world: &'w SecondaryWorld<D::Entity>) -> Self {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        super::assert_borrow::<Q>();
        crate::secondary::query::assert_borrow::<D::Entity, J>();

        Self {
            data,
            fetch: data.fetch(),
            secondary_fetch: <J::Fetch<'w> as SecondaryFetch<D::Entity>>::new(secondary_world),
            _phantom: PhantomData,
        }
    }
}

// TODO: `get` and `get_mut` for JoinQueryBorrow

impl<'w, Q, J, D> IntoIterator for JoinQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: SecondaryQuery<D::Entity>,
    D: WorldData,
{
    type Item = (
        QueryItem<'w, 'w, Q>,
        SecondaryQueryItem<'w, 'w, J, D::Entity>,
    );

    type IntoIter = JoinQueryFetchIter<'w, Q::Fetch<'w>, J::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: TODO
        let query_iter = unsafe { WorldFetchIter::new(self.data) };

        JoinQueryFetchIter {
            query_iter,
            secondary_fetch: self.secondary_fetch.clone(),
        }
    }
}

impl<'w, Q, J, D> JoinQueryBorrow<'w, Q, J, D>
where
    Q: QueryShared,
    J: SecondaryQueryShared<D::Entity>,
    D: WorldData,
{
    #[inline]
    pub fn get<'a, E>(
        &'a self,
        id: EntityId<E>,
    ) -> Option<(
        QueryItem<'w, 'a, Q>,
        SecondaryQueryItem<'w, 'a, J, D::Entity>,
    )>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        let item = unsafe { self.fetch.get(id.get()) }?;
        let secondary_fetch = self.secondary_fetch.as_ref()?;
        let secondary_item = unsafe { secondary_fetch.get(id)? };

        Some((item, secondary_item))
    }
}

impl<'w, Q, J, D> JoinQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: SecondaryQuery<D::Entity>,
    D: WorldData,
{
    #[inline]
    pub fn get_mut<'a, E>(
        &'a mut self,
        id: EntityId<E>,
    ) -> Option<(
        QueryItem<'w, 'a, Q>,
        SecondaryQueryItem<'w, 'a, J, D::Entity>,
    )>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        let item = unsafe { self.fetch.get(id.get()) }?;
        let secondary_fetch = self.secondary_fetch.as_ref()?;
        let secondary_item = unsafe { secondary_fetch.get(id)? };

        Some((item, secondary_item))
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
        let secondary_fetch = self.secondary_fetch.as_ref()?;

        loop {
            let (id, item) = self.query_iter.next()?;

            // Safety: `FetchIter` does not generate duplicate IDs.
            if let Some(secondary_item) = unsafe { secondary_fetch.get(id) } {
                return Some((item, secondary_item));
            }
        }
    }
}
