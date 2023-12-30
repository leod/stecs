use std::marker::PhantomData;

use crate::{entity::EntityVariant, world::WorldFetch, Entity, Id, Query, WorldData};

use super::{fetch::Fetch, iter::WorldFetchIter};

pub struct Nest2QueryBorrow<'w, Q, J0, J1, D> {
    pub(crate) data: &'w D,
    pub(crate) _phantom: PhantomData<(Q, J0, J1)>,
}

pub struct Nest1<'w, J1, D>
where
    J1: Fetch + 'w,
    D: WorldData + 'w,
{
    pub(crate) data: &'w D,
    pub(crate) ignore_ids: [Id<D::Entity>; 2],
    pub(crate) fetch1: D::Fetch<'w, J1>,
}

pub struct Nest2<'w, J0, J1, D>
where
    J0: Fetch + 'w,
    J1: Fetch + 'w,
    D: WorldData + 'w,
{
    pub(crate) data: &'w D,
    pub(crate) ignore_id: Id<D::Entity>,
    pub(crate) fetch0: D::Fetch<'w, J0>,
    pub(crate) fetch1: D::Fetch<'w, J1>,
}

pub struct Nest2DataFetchIter<'w, F, J0, J1, D>
where
    F: Fetch,
    J0: Fetch + 'w,
    J1: Fetch + 'w,
    D: WorldData,
{
    data: &'w D,
    query_iter: WorldFetchIter<'w, (<D::Entity as Entity>::FetchId<'w>, F), D>,
    nest_fetch0: D::Fetch<'w, J0>,
    nest_fetch1: D::Fetch<'w, J1>,
}

impl<'w, Q, J0, J1, D> IntoIterator for Nest2QueryBorrow<'w, Q, J0, J1, D>
where
    Q: Query,
    J0: Query,
    J1: Query,
    D: WorldData,
{
    type Item = (
        <Q::Fetch<'w> as Fetch>::Item<'w>,
        Nest2<'w, J0::Fetch<'w>, J1::Fetch<'w>, D>,
    );

    type IntoIter = Nest2DataFetchIter<'w, Q::Fetch<'w>, J0::Fetch<'w>, J1::Fetch<'w>, D>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: TODO
        let query_iter = unsafe { WorldFetchIter::new(self.data) };
        let nest_fetch0 = self.data.fetch();
        let nest_fetch1 = self.data.fetch();

        Nest2DataFetchIter {
            data: self.data,
            query_iter,
            nest_fetch0,
            nest_fetch1,
        }
    }
}

impl<'w, F, J0, J1, D> Iterator for Nest2DataFetchIter<'w, F, J0, J1, D>
where
    F: Fetch + 'w,
    J0: Fetch + 'w,
    J1: Fetch + 'w,
    D: WorldData,
{
    type Item = (<F as Fetch>::Item<'w>, Nest2<'w, J0, J1, D>);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let (id, item) = self.query_iter.next()?;
        let nest = Nest2 {
            data: self.data,
            ignore_id: id,
            fetch0: self.nest_fetch0.clone(),
            fetch1: self.nest_fetch1.clone(),
        };

        Some((item, nest))
    }
}

impl<'w, J0, J1, D> Nest2<'w, J0, J1, D>
where
    J0: Fetch,
    J1: Fetch,
    D: WorldData + 'w,
{
    // This has to take an exclusive `self` reference to prevent violating
    // Rust's borrowing rules if `J0` contains an exclusive borrow, since
    // `get()` could be called multiple times with the same `id`.
    #[inline]
    pub fn get_mut<'a, E>(&'a mut self, id: Id<E>) -> Option<(J0::Item<'a>, Nest1<'w, J1, D>)>
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

        let nest1 = Nest1 {
            data: self.data,
            ignore_ids: [self.ignore_id, id],
            fetch1: self.fetch1.clone(),
        };

        // Safety: TODO
        unsafe { self.fetch0.get(id.get()) }.map(|item| (item, nest1))
    }
}

impl<'w, J1, D> Nest1<'w, J1, D>
where
    J1: Fetch,
    D: WorldData + 'w,
{
    // This has to take an exclusive `self` reference to prevent violating
    // Rust's borrowing rules if `J1` contains an exclusive borrow, since
    // `get()` could be called multiple times with the same `id`.
    #[inline]
    pub fn get_mut<'a, E>(&'a mut self, id: Id<E>) -> Option<J1::Item<'a>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        // Safety: Do not allow borrowing the entity that the iterator that
        // produced `self` currently points to.
        if self.ignore_ids.contains(&id) {
            // TODO: Consider panicking. Design question.
            return None;
        }

        // Safety: TODO
        unsafe { self.fetch1.get(id.get()) }
    }
}

// TODO: Iterators
