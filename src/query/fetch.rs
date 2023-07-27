use std::marker::PhantomData;

use crate::{
    column::{Column, ColumnRawParts, ColumnRawPartsMut},
    entity::Columns,
    Component, WorldData,
};

use super::borrow_checker::BorrowChecker;

// TODO: unsafe maybe not needed.
pub unsafe trait Fetch: Copy {
    type Item<'f>
    where
        Self: 'f;

    fn new<A: Columns>(ids: &Column<thunderdome::Index>, columns: &A) -> Option<Self>;

    fn len(&self) -> usize;

    /// Fetches the components specified by `Self::Query` for the entity stored
    /// at `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index >= self.len()`.
    ///
    /// # Safety
    ///
    /// This is unsafe because it shifts the burden of checking Rust's borrowing
    /// rules to the caller. In particular, the caller has to ensure that this
    /// method is not called on an `index` whose components are already borrowed
    /// elsewhere (be it through `self` or not through `self`).
    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f;

    #[doc(hidden)]
    fn check_borrows(checker: &mut BorrowChecker);

    #[doc(hidden)]
    fn filter_by_outer<DOuter: WorldData>(_: &mut Option<Self>) {}
}

unsafe impl<C> Fetch for ColumnRawParts<C>
where
    C: Component,
{
    type Item<'f> = &'f C where Self: 'f;

    fn new<A: Columns>(_: &Column<thunderdome::Index>, columns: &A) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow().as_raw_parts())
    }

    fn len(&self) -> usize {
        self.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        assert!(index < <Self as Fetch>::len(self));

        unsafe { &*self.ptr.add(index) }
    }

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow::<C>();
    }
}

unsafe impl<C> Fetch for ColumnRawPartsMut<C>
where
    C: Component,
{
    type Item<'f> = &'f mut C where Self: 'f;

    fn new<A: Columns>(_: &Column<thunderdome::Index>, columns: &A) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow_mut().as_raw_parts_mut())
    }

    fn len(&self) -> usize {
        self.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        assert!(index < <Self as Fetch>::len(self));

        unsafe { &mut *self.ptr.add(index) }
    }

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow_mut::<C>();
    }
}

unsafe impl<F0, F1> Fetch for (F0, F1)
where
    F0: Fetch,
    F1: Fetch,
{
    type Item<'f> = (F0::Item<'f>, F1::Item<'f>) where Self: 'f;

    fn new<A: Columns>(ids: &Column<thunderdome::Index>, columns: &A) -> Option<Self> {
        let f0 = F0::new(ids, columns)?;
        let f1 = F1::new(ids, columns)?;

        assert_eq!(f0.len(), f1.len());

        Some((f0, f1))
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        (self.0.get(index), self.1.get(index))
    }

    fn check_borrows(checker: &mut BorrowChecker) {
        F0::check_borrows(checker);
        F1::check_borrows(checker);
    }
}

#[derive(Clone, Copy)]
pub struct FetchWith<F, R> {
    fetch: F,
    _phantom: PhantomData<R>,
}

unsafe impl<F, R> Fetch for FetchWith<F, R>
where
    F: Fetch,
    R: Fetch,
{
    type Item<'f> = F::Item<'f> where Self: 'f;

    fn new<A: Columns>(ids: &Column<thunderdome::Index>, columns: &A) -> Option<Self> {
        let fetch = F::new(ids, columns)?;

        R::new(ids, columns)?;

        Some(Self {
            fetch,
            _phantom: PhantomData,
        })
    }

    fn len(&self) -> usize {
        self.fetch.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        self.fetch.get(index)
    }

    fn check_borrows(checker: &mut BorrowChecker) {
        F::check_borrows(checker);
    }
}

#[derive(Clone, Copy)]
pub struct FetchWithout<F, R> {
    fetch: F,
    _phantom: PhantomData<R>,
}

unsafe impl<F, R> Fetch for FetchWithout<F, R>
where
    F: Fetch,
    R: Fetch,
{
    type Item<'f> = F::Item<'f> where Self: 'f;

    fn new<A: Columns>(ids: &Column<thunderdome::Index>, columns: &A) -> Option<Self> {
        let fetch = F::new(ids, columns)?;

        if R::new(ids, columns).is_some() {
            return None;
        }

        Some(Self {
            fetch,
            _phantom: PhantomData,
        })
    }

    fn len(&self) -> usize {
        self.fetch.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        self.fetch.get(index)
    }

    fn check_borrows(checker: &mut BorrowChecker) {
        F::check_borrows(checker);
    }
}

/*
pub struct FetchAnyEntityId<S: ArchetypeSet> {
    ids_raw_parts: ColumnRawParts<thunderdome::Index>,
    id_to_any_entity_id: fn(thunderdome::Index) -> S::AnyEntityId,
}

impl<S: ArchetypeSet> Clone for FetchAnyEntityId<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: ArchetypeSet> Copy for FetchAnyEntityId<S> {}

unsafe impl<S: ArchetypeSet> Fetch for FetchAnyEntityId<S> {
    type Item<'f> = S::AnyEntityId where Self: 'f;

    fn len(&self) -> usize {
        self.ids_raw_parts.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        assert!(index < <Self as Fetch>::len(self));

        let id = unsafe { *self.ids_raw_parts.ptr.add(index) };

        (self.id_to_any_entity_id)(id)
    }

    fn check_borrows(checker: &mut BorrowChecker) {}
}

unsafe impl<S: ArchetypeSet> FetchFromSet<S> for FetchAnyEntityId<S> {
    fn new<E: InArchetypeSet<S>>(ids: &Column<thunderdome::Index>, _: &E::Columns) -> Option<Self> {
        Some(Self {
            ids_raw_parts: ids.as_raw_parts(),
            id_to_any_entity_id: |id| E::any_entity_id(E::entity_id(id)),
        })
    }
}

pub struct FetchEntityId<E> {
    ids_raw_parts: ColumnRawParts<thunderdome::Index>,
    _phantom: PhantomData<E>,
}

impl<E> Clone for FetchEntityId<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for FetchEntityId<E> {}

unsafe impl<E: Entity> Fetch for FetchEntityId<E> {
    type Item<'f> = EntityId<E> where Self: 'f;

    fn len(&self) -> usize {
        self.ids_raw_parts.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        assert!(index < <Self as Fetch>::len(self));

        let id = unsafe { *self.ids_raw_parts.ptr.add(index) };

        EntityId::new_unchecked(id)
    }

    fn check_borrows(checker: &mut BorrowChecker) {}
}

unsafe impl<E, S> FetchFromSet<S> for FetchEntityId<E>
where
    E: Entity,
    S: ArchetypeSet,
{
    fn new<F: InArchetypeSet<S>>(ids: &Column<thunderdome::Index>, _: &F::Columns) -> Option<Self> {
        Some(Self {
            ids_raw_parts: ids.as_raw_parts(),
            _phantom: PhantomData,
        })
    }
}

*/
