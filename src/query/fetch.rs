use std::marker::PhantomData;

use crate::{
    column::{ColumnRawParts, ColumnRawPartsMut},
    ArchetypeSet, Column, Component, Entity, EntityColumns, InArchetypeSet,
};

pub unsafe trait Fetch<'w>: Copy + 'w {
    type Item<'f>
    where
        'w: 'f;

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
        'w: 'f;
}

// TODO: 'w maybe not needed.
pub unsafe trait FetchFromSet<'w, S: ArchetypeSet>: Fetch<'w> {
    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &'w Column<thunderdome::Index>,
        columns: &'w E::Columns,
    ) -> Option<Self>;
}

unsafe impl<'w, C> Fetch<'w> for ColumnRawParts<C>
where
    C: Component,
{
    type Item<'f> = &'f C where 'w: 'f;

    fn len(&self) -> usize {
        self.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f,
    {
        assert!(index < <Self as Fetch>::len(self));

        unsafe { &*self.ptr.add(index) }
    }
}

unsafe impl<'w, C, S> FetchFromSet<'w, S> for ColumnRawParts<C>
where
    C: Component,
    S: ArchetypeSet,
{
    fn new<E: InArchetypeSet<S>>(
        _: &Column<thunderdome::Index>,
        columns: &'w E::Columns,
    ) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow().as_raw_parts())
    }
}

unsafe impl<'w, C> Fetch<'w> for ColumnRawPartsMut<C>
where
    C: Component,
{
    type Item<'f> = &'f mut C where 'w: 'f;

    fn len(&self) -> usize {
        self.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f,
    {
        assert!(index < <Self as Fetch>::len(self));

        unsafe { &mut *self.ptr.add(index) }
    }
}

unsafe impl<'w, C, S> FetchFromSet<'w, S> for ColumnRawPartsMut<C>
where
    C: Component,
    S: ArchetypeSet,
{
    fn new<E: Entity>(_: &'w Column<thunderdome::Index>, columns: &'w E::Columns) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow_mut().as_raw_parts_mut())
    }
}

unsafe impl<'w, F0, F1> Fetch<'w> for (F0, F1)
where
    F0: Fetch<'w>,
    F1: Fetch<'w>,
{
    type Item<'f> = (F0::Item<'f>, F1::Item<'f>) where 'w: 'f;

    fn len(&self) -> usize {
        self.0.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f,
    {
        (self.0.get(index), self.1.get(index))
    }
}

unsafe impl<'w, F0, F1, S> FetchFromSet<'w, S> for (F0, F1)
where
    F0: FetchFromSet<'w, S>,
    F1: FetchFromSet<'w, S>,
    S: ArchetypeSet,
{
    fn new<E: InArchetypeSet<S>>(
        untypes_keys: &'w Column<thunderdome::Index>,
        columns: &'w E::Columns,
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
        Self {
            raw_parts: self.raw_parts.clone(),
            untyped_key_to_id: self.untyped_key_to_id.clone(),
        }
    }
}

impl<S> Copy for FetchEntityId<S> where S: ArchetypeSet {}

unsafe impl<'w, S> Fetch<'w> for FetchEntityId<S>
where
    S: ArchetypeSet + 'w,
{
    type Item<'f> = S::EntityId where 'w: 'f;

    fn len(&self) -> usize {
        self.raw_parts.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f,
    {
        assert!(index < <Self as Fetch>::len(self));

        let untyped_key = unsafe { *self.raw_parts.ptr.add(index) };

        (self.untyped_key_to_id)(untyped_key)
    }
}

unsafe impl<'w, S> FetchFromSet<'w, S> for FetchEntityId<S>
where
    S: ArchetypeSet + 'w,
{
    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        _: &'w E::Columns,
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

unsafe impl<'w, F, R> Fetch<'w> for FetchWith<F, R>
where
    F: Fetch<'w>,
    R: Fetch<'w>,
{
    type Item<'f> = F::Item<'f> where 'w: 'f;

    fn len(&self) -> usize {
        self.fetch.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f,
    {
        self.fetch.get(index)
    }
}

unsafe impl<'w, F, R, S> FetchFromSet<'w, S> for FetchWith<F, R>
where
    F: FetchFromSet<'w, S>,
    R: FetchFromSet<'w, S>,
    S: ArchetypeSet,
{
    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &'w Column<thunderdome::Index>,
        columns: &'w E::Columns,
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

unsafe impl<'w, F, R> Fetch<'w> for FetchWithout<F, R>
where
    F: Fetch<'w>,
    R: Fetch<'w>,
{
    type Item<'f> = F::Item<'f> where 'w: 'f;

    fn len(&self) -> usize {
        self.fetch.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f,
    {
        self.fetch.get(index)
    }
}

unsafe impl<'w, F, R, S> FetchFromSet<'w, S> for FetchWithout<F, R>
where
    F: FetchFromSet<'w, S>,
    R: FetchFromSet<'w, S>,
    S: ArchetypeSet,
{
    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &'w Column<thunderdome::Index>,
        columns: &'w E::Columns,
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
