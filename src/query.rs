mod iter;
mod stream;

use std::{any::type_name, marker::PhantomData};

use crate::{
    archetype_set::ArchetypeSetFetch,
    borrow_checker::BorrowChecker,
    column::{ColumnRawParts, ColumnRawPartsMut},
    ArchetypeSet, Column, Component, Entity, EntityColumns, InArchetypeSet,
};

pub trait Fetch<'a, S: ArchetypeSet>: Copy {
    type Item: 'a;

    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        columns: &'a E::Columns,
    ) -> Option<Self>;

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
    unsafe fn get(&self, index: usize) -> Self::Item;
}

impl<'a, C, S> Fetch<'a, S> for ColumnRawParts<C>
where
    C: Component,
    S: ArchetypeSet,
{
    type Item = &'a C;

    fn new<E: InArchetypeSet<S>>(
        _: &Column<thunderdome::Index>,
        columns: &'a E::Columns,
    ) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow().as_raw_parts())
    }

    fn len(&self) -> usize {
        self.len
    }

    unsafe fn get(&self, index: usize) -> Self::Item {
        assert!(index < <Self as Fetch<S>>::len(self));

        unsafe { &*self.ptr.add(index) }
    }
}

impl<'a, C, S> Fetch<'a, S> for ColumnRawPartsMut<C>
where
    C: Component,
    S: ArchetypeSet,
{
    type Item = &'a mut C;

    fn new<E: Entity>(_: &Column<thunderdome::Index>, columns: &'a E::Columns) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow_mut().as_raw_parts_mut())
    }

    fn len(&self) -> usize {
        self.len
    }

    unsafe fn get(&self, index: usize) -> Self::Item {
        assert!(index < <Self as Fetch<S>>::len(self));

        unsafe { &mut *self.ptr.add(index) }
    }
}

impl<'a, F0, F1, S> Fetch<'a, S> for (F0, F1)
where
    F0: Fetch<'a, S>,
    F1: Fetch<'a, S>,
    S: ArchetypeSet,
{
    type Item = (F0::Item, F1::Item);

    fn new<E: InArchetypeSet<S>>(
        untypes_keys: &Column<thunderdome::Index>,
        columns: &'a E::Columns,
    ) -> Option<Self> {
        let f0 = F0::new::<E>(untypes_keys, columns)?;
        let f1 = F1::new::<E>(untypes_keys, columns)?;

        assert_eq!(f0.len(), f1.len());

        Some((f0, f1))
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    unsafe fn get(&self, index: usize) -> Self::Item {
        (self.0.get(index), self.1.get(index))
    }
}

#[derive(Clone, Copy)]
pub struct FetchEntityId<EntityId> {
    raw_parts: ColumnRawParts<thunderdome::Index>,
    untyped_key_to_id: fn(thunderdome::Index) -> EntityId,
}

impl<'a, S> Fetch<'a, S> for FetchEntityId<S::EntityId>
where
    S: ArchetypeSet,
{
    type Item = S::EntityId;

    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        _: &'a E::Columns,
    ) -> Option<Self> {
        Some(Self {
            raw_parts: untyped_keys.as_raw_parts(),
            untyped_key_to_id: |key| E::key_to_id(E::untyped_key_to_key(key)),
        })
    }

    fn len(&self) -> usize {
        self.raw_parts.len
    }

    unsafe fn get(&self, index: usize) -> Self::Item {
        assert!(index < <Self as Fetch<S>>::len(self));

        let untyped_key = unsafe { *self.raw_parts.ptr.add(index) };

        (self.untyped_key_to_id)(untyped_key)
    }
}

pub trait Query<'a, S: ArchetypeSet> {
    type Fetch: Fetch<'a, S>;

    #[doc(hidden)]
    fn check_borrows(checker: &mut BorrowChecker);
}

#[derive(Clone, Copy)]
pub struct FetchWith<F, R> {
    fetch: F,
    _phantom: PhantomData<R>,
}

impl<'a, F, R, S> Fetch<'a, S> for FetchWith<F, R>
where
    F: Fetch<'a, S>,
    R: Fetch<'a, S>,
    S: ArchetypeSet,
{
    type Item = F::Item;

    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        columns: &'a E::Columns,
    ) -> Option<Self> {
        let fetch = F::new::<E>(untyped_keys, columns)?;

        R::new::<E>(untyped_keys, columns)?;

        Some(Self {
            fetch,
            _phantom: PhantomData,
        })
    }

    fn len(&self) -> usize {
        self.fetch.len()
    }

    unsafe fn get(&self, index: usize) -> Self::Item {
        self.fetch.get(index)
    }
}

#[derive(Clone, Copy)]
pub struct FetchWithout<F, R> {
    fetch: F,
    _phantom: PhantomData<R>,
}

impl<'a, F, R, S> Fetch<'a, S> for FetchWithout<F, R>
where
    F: Fetch<'a, S>,
    R: Fetch<'a, S>,
    S: ArchetypeSet,
{
    type Item = F::Item;

    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        columns: &'a E::Columns,
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

    fn len(&self) -> usize {
        self.fetch.len()
    }

    unsafe fn get(&self, index: usize) -> Self::Item {
        self.fetch.get(index)
    }
}

impl<'a, C, S> Query<'a, S> for &'a C
where
    C: Component,
    S: ArchetypeSet,
{
    type Fetch = ColumnRawParts<C>;

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow::<C>();
    }
}

impl<'a, C, S> Query<'a, S> for &'a mut C
where
    C: Component,
    S: ArchetypeSet,
{
    type Fetch = ColumnRawPartsMut<C>;

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow_mut::<C>();
    }
}

impl<'a, Q0, Q1, S> Query<'a, S> for (Q0, Q1)
where
    Q0: Query<'a, S>,
    Q1: Query<'a, S>,
    S: ArchetypeSet,
{
    type Fetch = (Q0::Fetch, Q1::Fetch);

    fn check_borrows(checker: &mut BorrowChecker) {
        Q0::check_borrows(checker);
        Q1::check_borrows(checker);
    }
}

pub struct With<Q, R>(PhantomData<(Q, R)>);

impl<'a, Q, R, S> Query<'a, S> for With<Q, R>
where
    Q: Query<'a, S>,
    R: Query<'a, S>,
    S: ArchetypeSet,
{
    type Fetch = FetchWith<Q::Fetch, R::Fetch>;

    fn check_borrows(checker: &mut BorrowChecker) {
        Q::check_borrows(checker);
    }
}

pub struct Without<Q, R>(PhantomData<(Q, R)>);

impl<'a, Q, R, S> Query<'a, S> for Without<Q, R>
where
    Q: Query<'a, S>,
    R: Query<'a, S>,
    S: ArchetypeSet,
{
    type Fetch = FetchWithout<Q::Fetch, R::Fetch>;

    fn check_borrows(checker: &mut BorrowChecker) {
        Q::check_borrows(checker);
    }
}

// Safety: Before constructing a `FetchIter`, use `BorrowChecker` to ensure that
// the query does not specify borrows that violate Rust's borrowing rules. Also,
// do not allow constructing references to the entity at which the `FetchIter`
// currently points.
pub struct FetchIter<'a, F, S> {
    i: usize,
    fetch: F,
    _phantom: PhantomData<&'a S>,
}

impl<'a, F, S> FetchIter<'a, F, S> {
    pub fn new(fetch: F) -> Self {
        Self {
            i: 0,
            fetch,
            _phantom: PhantomData,
        }
    }
}

impl<'a, F, S> Iterator for FetchIter<'a, F, S>
where
    F: Fetch<'a, S>,
    S: ArchetypeSet,
{
    type Item = F::Item;

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

pub struct ArchetypeSetFetchIter<'a, F, S>
where
    F: ArchetypeSetFetch<'a, S>,
    S: ArchetypeSet,
{
    archetype_set_iter: F::Iter,
    current_fetch_iter: Option<FetchIter<'a, F::Fetch, S>>,
}

impl<'a, F, S> Iterator for ArchetypeSetFetchIter<'a, F, S>
where
    F: ArchetypeSetFetch<'a, S>,
    S: ArchetypeSet,
{
    type Item = <F::Fetch as Fetch<'a, S>>::Item;

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
            if self.current_fetch_iter.is_none() {
                return None;
            }
        }
    }
}

pub struct QueryResult<'a, Q, S> {
    archetype_set: &'a mut S,
    _phantom: PhantomData<Q>,
}

impl<'a, Q, S> IntoIterator for QueryResult<'a, Q, S>
where
    Q: Query<'a, S>,
    S: ArchetypeSet,
{
    type Item = <Q::Fetch as Fetch<'a, S>>::Item;

    type IntoIter = ArchetypeSetFetchIter<'a, S::Fetch<'a, Q::Fetch>, S>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        Q::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: A `QueryResult` exclusively borrows the `archetype_set: &'a
        // mut S`. Also, `into_iter` consumes the `QueryResult` while
        // maintaining the lifetime `'a`. Thus, it is not possible to construct
        // references to entities in `archetype_set` outside of the returned
        // iterator, thereby satisfying the requirement of `FetchIter`.
        let mut archetype_set_iter = self.archetype_set.fetch::<Q::Fetch>().iter();

        let current_fetch_iter = archetype_set_iter.next().map(FetchIter::new);

        ArchetypeSetFetchIter {
            archetype_set_iter,
            current_fetch_iter,
        }
    }
}

impl<'a, Q, S> QueryResult<'a, Q, S> {
    pub(crate) fn new(archetype_set: &'a mut S) -> Self {
        Self {
            archetype_set,
            _phantom: PhantomData,
        }
    }

    pub fn with<R>(self) -> QueryResult<'a, With<Q, R>, S> {
        QueryResult::new(self.archetype_set)
    }

    pub fn without<R>(self) -> QueryResult<'a, Without<Q, R>, S> {
        QueryResult::new(self.archetype_set)
    }
}
