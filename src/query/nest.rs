use std::{any::type_name, marker::PhantomData};

use crate::{entity::EntityVariant, world::WorldFetch, Entity, EntityId, Query, WorldData};

use super::{borrow_checker::BorrowChecker, fetch::Fetch, iter::WorldFetchIter};

pub struct NestQueryBorrow<'w, Q, J, D> {
    pub(crate) data: &'w D,
    pub(crate) _phantom: PhantomData<(Q, J)>,
}

impl<'w, Q, J, D> NestQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: Query,
    D: WorldData,
{
    pub fn get_mut<'f, E>(
        &mut self,
        id: EntityId<E>,
    ) -> Option<(<Q::Fetch<'f> as Fetch>::Item<'f>, Nest<'f, J::Fetch<'f>, D>)>
    where
        'w: 'f,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        // TODO: Cache?

        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'f> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));
        <J::Fetch<'f> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<J>()));

        let world_fetch = self.data.fetch::<Q::Fetch<'f>>();

        // Safety: TODO
        let item = unsafe { world_fetch.get(id.get()) }?;
        let nest = Nest {
            data: self.data,
            ignore_id: id,
            fetch: self.data.fetch::<J::Fetch<'f>>(),
        };

        Some((item, nest))
    }
}

// TODO: Implement `get` for `NestQueryResult`.

impl<'w, Q, J, D> IntoIterator for NestQueryBorrow<'w, Q, J, D>
where
    Q: Query,
    J: Query,
    D: WorldData,
{
    type Item = (<Q::Fetch<'w> as Fetch>::Item<'w>, Nest<'w, J::Fetch<'w>, D>);

    type IntoIter = NestDataFetchIter<'w, Q::Fetch<'w>, J::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));
        <J::Fetch<'w> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<J>()));

        // Safety: TODO
        let query_iter = unsafe { WorldFetchIter::new(self.data) };
        let nest_fetch = self.data.fetch();

        NestDataFetchIter {
            data: self.data,
            query_iter,
            nest_fetch,
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
    pub(crate) fetch: D::Fetch<'w, J>,
}

pub struct NestDataFetchIter<'w, F, J, D>
where
    F: Fetch,
    J: Fetch + 'w,
    D: WorldData,
{
    data: &'w D,
    query_iter: WorldFetchIter<'w, (<D::Entity as Entity>::FetchId<'w>, F), D>,
    nest_fetch: D::Fetch<'w, J>,
}

impl<'w, F, J, D> Iterator for NestDataFetchIter<'w, F, J, D>
where
    F: Fetch + 'w,
    J: Fetch + 'w,
    D: WorldData,
{
    type Item = (<F as Fetch>::Item<'w>, Nest<'w, J, D>);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, item) = self.query_iter.next()?;
        let nest = Nest {
            data: self.data,
            ignore_id: id,
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
    pub fn get_mut<'f, E>(&'f mut self, id: EntityId<E>) -> Option<J::Item<'f>>
    where
        'w: 'f,
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
        unsafe { self.fetch.get(id.get()) }
    }
}

pub struct NestIter<'w, J, D>
where
    J: Fetch + 'w,
    D: WorldData + 'w,
{
    ignore_id: EntityId<D::Entity>,
    id_iter: WorldFetchIter<'w, <D::Entity as Entity>::FetchId<'w>, D>,
    data_iter: WorldFetchIter<'w, J, D>,
}

impl<'w, J, D> Iterator for NestIter<'w, J, D>
where
    J: Fetch + 'w,
    D: WorldData + 'w,
{
    type Item = J::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(mut id) = self.id_iter.next() else {
            self.data_iter.next();
            return None;
        };

        // At this point, `id_iter` has been advanced one more time than
        // `data_iter`.

        while id == self.ignore_id {
            // Safety: We are viewing the entity that is to be ignored, so we
            // must *not* call `next()` instead of `skip_one()`, since that
            // could create an aliasing reference. Instead, we just let the
            // pointers skip over the current entity.
            self.data_iter.skip_one();

            let next_id = self.id_iter.next();

            let Some(next_id) = next_id else {
                self.data_iter.next();
                return None;
            };

            id = next_id;
        }

        // Safety: Again, `id_iter` has been advanced one more time than
        // `data_iter`, and now we now know that they `id` does not point to the
        // entity that is to be ignored, so it is safe to call `next()` on
        // `data_iter`.
        self.data_iter.next()
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
        // Safety: Ids cannot be mutably borrowed, so there is no invalid aliasing.
        let id_iter = unsafe { WorldFetchIter::new(self.data) };

        // Safety: TODO
        let data_iter = unsafe { WorldFetchIter::new(self.data) };

        NestIter {
            ignore_id: self.ignore_id,
            id_iter,
            data_iter,
        }
    }
}

// TODO: Size hints for iterators
