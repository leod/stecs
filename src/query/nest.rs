use std::marker::PhantomData;

use crate::{entity::EntityVariant, world::WorldFetch, Entity, EntityId, Query, WorldData};

use super::{
    assert_borrow, fetch::Fetch, iter::WorldFetchIter, nest2::Nest2QueryBorrow, QueryItem,
};

pub struct NestQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: Query,
    D: WorldData,
{
    data: &'w D,
    world_fetch_q: D::Fetch<'w, Q::Fetch<'w>>,
    world_fetch_j: D::Fetch<'w, J::Fetch<'w>>,
    _phantom: PhantomData<(Q, J)>,
}

impl<'w, Q, J, D> NestQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: Query,
    D: WorldData,
{
    pub(crate) fn new(data: &'w D) -> Self {
        // Safety: The query must satisfy Rust's borrowing rules.
        assert_borrow::<Q>();
        assert_borrow::<J>();

        Self {
            data,
            world_fetch_q: data.fetch(),
            world_fetch_j: data.fetch(),
            _phantom: PhantomData,
        }
    }
}

impl<'w, Q, J, D> NestQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: Query,
    D: WorldData,
{
    pub fn get_mut<'a, E>(
        &'a mut self,
        id: EntityId<E>,
    ) -> Option<(QueryItem<'w, 'a, Q>, Nest<'w, J::Fetch<'w>, D>)>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();
        let item = unsafe { self.world_fetch_q.get(id.get()) }?;

        let nest = Nest {
            data: self.data,
            ignore_id: id,
            world_fetch_j: self.world_fetch_j.clone(),
        };

        Some((item, nest))
    }

    pub fn nest<J1>(self) -> Nest2QueryBorrow<'w, Q, J, J1, D>
    where
        J1: Query,
    {
        // Safety: The query must satisfy Rust's borrowing rules.
        assert_borrow::<J1>();

        Nest2QueryBorrow {
            data: self.data,
            _phantom: PhantomData,
        }
    }
}

impl<'w, Q, J, D> IntoIterator for NestQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: Query,
    D: WorldData,
{
    type Item = (<Q::Fetch<'w> as Fetch>::Item<'w>, Nest<'w, J::Fetch<'w>, D>);

    type IntoIter = NestDataFetchIter<'w, Q::Fetch<'w>, J::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: TODO
        let world_iter_q = unsafe { WorldFetchIter::new(self.data) };

        NestDataFetchIter {
            data: self.data,
            world_iter_q,
            world_fetch_j: self.world_fetch_j,
        }
    }
}

pub struct Nest<'w, J, D>
where
    J: Fetch + 'w,
    D: WorldData + 'w,
{
    pub(crate) data: &'w D,
    pub(crate) ignore_id: EntityId<D::Entity>,
    pub(crate) world_fetch_j: D::Fetch<'w, J>,
}

pub struct NestDataFetchIter<'w, F, J, D>
where
    F: Fetch,
    J: Fetch + 'w,
    D: WorldData,
{
    data: &'w D,
    world_iter_q: WorldFetchIter<'w, (<D::Entity as Entity>::FetchId<'w>, F), D>,
    world_fetch_j: D::Fetch<'w, J>,
}

impl<'w, F, J, D> Iterator for NestDataFetchIter<'w, F, J, D>
where
    F: Fetch + 'w,
    J: Fetch + 'w,
    D: WorldData,
{
    type Item = (<F as Fetch>::Item<'w>, Nest<'w, J, D>);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, item) = self.world_iter_q.next()?;
        let nest = Nest {
            data: self.data,
            ignore_id: id,
            world_fetch_j: self.world_fetch_j.clone(),
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
    pub fn get_mut<'a, E>(&'a mut self, id: EntityId<E>) -> Option<J::Item<'a>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        // Safety: Do not allow borrowing the entity that the iterator that
        // produced `self` currently points to.
        if id == self.ignore_id {
            // TODO: Consider panicking. Design question.
            return None;
        }

        // Safety: TODO
        unsafe { self.world_fetch_j.get(id.get()) }
    }
}

pub struct NestIter<'w, J, D>
where
    J: Fetch + 'w,
    D: WorldData + 'w,
{
    ignore_id: EntityId<D::Entity>,
    iter_id: WorldFetchIter<'w, <D::Entity as Entity>::FetchId<'w>, D>,
    iter_j: WorldFetchIter<'w, J, D>,
}

impl<'w, J, D> Iterator for NestIter<'w, J, D>
where
    J: Fetch + 'w,
    D: WorldData + 'w,
{
    type Item = J::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(mut id) = self.iter_id.next() else {
            self.iter_j.next();
            return None;
        };

        // At this point, `id_iter` has been advanced one more time than
        // `data_iter`.

        while id == self.ignore_id {
            // Safety: We are viewing the entity that is to be ignored, so we
            // must *not* call `next()` instead of `skip_one()`, since that
            // could create an aliasing reference. Instead, we just let the
            // pointers skip over the current entity.
            self.iter_j.skip_one();

            let next_id = self.iter_id.next();

            let Some(next_id) = next_id else {
                self.iter_j.next();
                return None;
            };

            id = next_id;
        }

        // Safety: Again, `id_iter` has been advanced one more time than
        // `data_iter`, and now we now know that they `id` does not point to the
        // entity that is to be ignored, so it is safe to call `next()` on
        // `data_iter`.
        self.iter_j.next()
    }
}

impl<'w, J, D> IntoIterator for Nest<'w, J, D>
where
    J: Fetch,
    D: WorldData + 'w,
{
    type Item = J::Item<'w>;

    type IntoIter = NestIter<'w, J, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Ids cannot be mutably queries, so there is no invalid
        // aliasing.
        let iter_id = unsafe { WorldFetchIter::new(self.data) };

        // Safety: TODO
        let iter_j = unsafe { WorldFetchIter::from_world_fetch(self.world_fetch_j) };

        NestIter {
            ignore_id: self.ignore_id,
            iter_id,
            iter_j,
        }
    }
}

// TODO: Size hints for iterators
