use std::marker::PhantomData;

use crate::{
    archetype::Columns,
    archetype_set::InArchetypeSet,
    column::{ColumnRawParts, ColumnRawPartsMut},
    internal::Column,
    ArchetypeSet, Component, Entity,
};

// TODO: unsafe maybe not needed.
pub unsafe trait Fetch: Copy {
    type Item<'f>
    where
        Self: 'f;

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
}

// TODO: 'w maybe not needed.
// TODO: unsafe maybe not needed.
pub unsafe trait FetchFromSet<S: ArchetypeSet>: Fetch {
    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        columns: &E::Columns,
    ) -> Option<Self>;
}

unsafe impl<C> Fetch for ColumnRawParts<C>
where
    C: Component,
{
    type Item<'f> = &'f C where Self: 'f;

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
}

unsafe impl<C, S> FetchFromSet<S> for ColumnRawParts<C>
where
    C: Component,
    S: ArchetypeSet,
{
    fn new<E: InArchetypeSet<S>>(
        _: &Column<thunderdome::Index>,
        columns: &E::Columns,
    ) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow().as_raw_parts())
    }
}

unsafe impl<C> Fetch for ColumnRawPartsMut<C>
where
    C: Component,
{
    type Item<'f> = &'f mut C where Self: 'f;

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
}

unsafe impl<C, S> FetchFromSet<S> for ColumnRawPartsMut<C>
where
    C: Component,
    S: ArchetypeSet,
{
    fn new<E: Entity>(_: &Column<thunderdome::Index>, columns: &E::Columns) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow_mut().as_raw_parts_mut())
    }
}

unsafe impl<F0, F1> Fetch for (F0, F1)
where
    F0: Fetch,
    F1: Fetch,
{
    type Item<'f> = (F0::Item<'f>, F1::Item<'f>) where Self: 'f;

    fn len(&self) -> usize {
        self.0.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        (self.0.get(index), self.1.get(index))
    }
}

unsafe impl<F0, F1, S> FetchFromSet<S> for (F0, F1)
where
    F0: FetchFromSet<S>,
    F1: FetchFromSet<S>,
    S: ArchetypeSet,
{
    fn new<E: InArchetypeSet<S>>(
        untypes_keys: &Column<thunderdome::Index>,
        columns: &E::Columns,
    ) -> Option<Self> {
        let f0 = F0::new::<E>(untypes_keys, columns)?;
        let f1 = F1::new::<E>(untypes_keys, columns)?;

        assert_eq!(f0.len(), f1.len());

        Some((f0, f1))
    }
}

pub struct FetchEntityId<S>
where
    S: ArchetypeSet,
{
    raw_parts: ColumnRawParts<thunderdome::Index>,
    untyped_key_to_id: fn(thunderdome::Index) -> S::EntityId,
}

impl<S> Clone for FetchEntityId<S>
where
    S: ArchetypeSet,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for FetchEntityId<S> where S: ArchetypeSet {}

unsafe impl<S> Fetch for FetchEntityId<S>
where
    S: ArchetypeSet,
{
    type Item<'f> = S::EntityId where Self: 'f;

    fn len(&self) -> usize {
        self.raw_parts.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        assert!(index < <Self as Fetch>::len(self));

        let untyped_key = unsafe { *self.raw_parts.ptr.add(index) };

        (self.untyped_key_to_id)(untyped_key)
    }
}

unsafe impl<S> FetchFromSet<S> for FetchEntityId<S>
where
    S: ArchetypeSet,
{
    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        _: &E::Columns,
    ) -> Option<Self> {
        Some(Self {
            raw_parts: untyped_keys.as_raw_parts(),
            untyped_key_to_id: |key| E::key_to_id(E::untyped_key_to_key(key)),
        })
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

    fn len(&self) -> usize {
        self.fetch.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        self.fetch.get(index)
    }
}

unsafe impl<F, R, S> FetchFromSet<S> for FetchWith<F, R>
where
    F: FetchFromSet<S>,
    R: FetchFromSet<S>,
    S: ArchetypeSet,
{
    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        columns: &E::Columns,
    ) -> Option<Self> {
        let fetch = F::new::<E>(untyped_keys, columns)?;

        R::new::<E>(untyped_keys, columns)?;

        Some(Self {
            fetch,
            _phantom: PhantomData,
        })
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

    fn len(&self) -> usize {
        self.fetch.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        self.fetch.get(index)
    }
}

unsafe impl<F, R, S> FetchFromSet<S> for FetchWithout<F, R>
where
    F: FetchFromSet<S>,
    R: FetchFromSet<S>,
    S: ArchetypeSet,
{
    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        columns: &E::Columns,
    ) -> Option<Self> {
        let fetch = F::new::<E>(untyped_keys, columns)?;

        if R::new::<E>(untyped_keys, columns).is_some() {
            return None;
        }

        Some(Self {
            fetch,
            _phantom: PhantomData,
        })
    }
}
