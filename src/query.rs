mod iter;
mod stream;

use std::{any::type_name, marker::PhantomData};

use crate::{
    archetype_set::ArchetypeSetFetch,
    borrow_checker::BorrowChecker,
    column::{ColumnRawParts, ColumnRawPartsMut},
    ArchetypeSet, Column, Component, Entity, EntityColumns, InArchetypeSet,
};

// TODO: 'w maybe not needed.
pub trait Fetch<'w, S: ArchetypeSet>: Copy + 'w {
    type Item<'f>
    where
        'w: 'f;

    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &'w Column<thunderdome::Index>,
        columns: &'w E::Columns,
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
    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f;
}

impl<'w, C, S> Fetch<'w, S> for ColumnRawParts<C>
where
    C: Component,
    S: ArchetypeSet,
{
    type Item<'f> = &'f C where 'w: 'f;

    fn new<E: InArchetypeSet<S>>(
        _: &Column<thunderdome::Index>,
        columns: &'w E::Columns,
    ) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow().as_raw_parts())
    }

    fn len(&self) -> usize {
        self.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f,
    {
        assert!(index < <Self as Fetch<S>>::len(self));

        unsafe { &*self.ptr.add(index) }
    }
}

impl<'w, C, S> Fetch<'w, S> for ColumnRawPartsMut<C>
where
    C: Component,
    S: ArchetypeSet,
{
    type Item<'f> = &'f mut C where 'w: 'f;

    fn new<E: Entity>(_: &'w Column<thunderdome::Index>, columns: &'w E::Columns) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.borrow_mut().as_raw_parts_mut())
    }

    fn len(&self) -> usize {
        self.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f,
    {
        assert!(index < <Self as Fetch<S>>::len(self));

        println!("getting {:?} + {}", self.ptr, index);

        unsafe { &mut *self.ptr.add(index) }
    }
}

impl<'w, F0, F1, S> Fetch<'w, S> for (F0, F1)
where
    F0: Fetch<'w, S>,
    F1: Fetch<'w, S>,
    S: ArchetypeSet,
{
    type Item<'f> = (F0::Item<'f>, F1::Item<'f>) where 'w: 'f;

    fn new<E: InArchetypeSet<S>>(
        untypes_keys: &'w Column<thunderdome::Index>,
        columns: &'w E::Columns,
    ) -> Option<Self> {
        let f0 = F0::new::<E>(untypes_keys, columns)?;
        let f1 = F1::new::<E>(untypes_keys, columns)?;

        assert_eq!(f0.len(), f1.len());

        Some((f0, f1))
    }

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

#[derive(Clone, Copy)]
pub struct FetchEntityId<EntityId> {
    raw_parts: ColumnRawParts<thunderdome::Index>,
    untyped_key_to_id: fn(thunderdome::Index) -> EntityId,
}

impl<'w, S> Fetch<'w, S> for FetchEntityId<S::EntityId>
where
    S: ArchetypeSet,
{
    type Item<'f> = S::EntityId where 'w: 'f;

    fn new<E: InArchetypeSet<S>>(
        untyped_keys: &Column<thunderdome::Index>,
        _: &'w E::Columns,
    ) -> Option<Self> {
        Some(Self {
            raw_parts: untyped_keys.as_raw_parts(),
            untyped_key_to_id: |key| E::key_to_id(E::untyped_key_to_key(key)),
        })
    }

    fn len(&self) -> usize {
        self.raw_parts.len
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        'w: 'f,
    {
        assert!(index < <Self as Fetch<S>>::len(self));

        let untyped_key = unsafe { *self.raw_parts.ptr.add(index) };

        (self.untyped_key_to_id)(untyped_key)
    }
}

#[derive(Clone, Copy)]
pub struct FetchWith<F, R> {
    fetch: F,
    _phantom: PhantomData<R>,
}

impl<'w, F, R, S> Fetch<'w, S> for FetchWith<F, R>
where
    F: Fetch<'w, S>,
    R: Fetch<'w, S>,
    S: ArchetypeSet,
{
    type Item<'f> = F::Item<'f> where 'w: 'f;

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

#[derive(Clone, Copy)]
pub struct FetchWithout<F, R> {
    fetch: F,
    _phantom: PhantomData<R>,
}

impl<'w, F, R, S> Fetch<'w, S> for FetchWithout<F, R>
where
    F: Fetch<'w, S>,
    R: Fetch<'w, S>,
    S: ArchetypeSet,
{
    type Item<'f> = F::Item<'f> where 'w: 'f;

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

pub trait Query<S: ArchetypeSet> {
    type Fetch<'w>: Fetch<'w, S>;

    #[doc(hidden)]
    fn check_borrows(checker: &mut BorrowChecker);
}

impl<'q, C, S> Query<S> for &'q C
where
    C: Component,
    S: ArchetypeSet,
{
    type Fetch<'w> = ColumnRawParts<C>;

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow::<C>();
    }
}

impl<'q, C, S> Query<S> for &'q mut C
where
    C: Component,
    S: ArchetypeSet,
{
    type Fetch<'w> = ColumnRawPartsMut<C>;

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow_mut::<C>();
    }
}

impl<'q, Q0, Q1, S> Query<S> for (Q0, Q1)
where
    Q0: Query<S>,
    Q1: Query<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = (Q0::Fetch<'w>, Q1::Fetch<'w>);

    fn check_borrows(checker: &mut BorrowChecker) {
        Q0::check_borrows(checker);
        Q1::check_borrows(checker);
    }
}

pub struct With<Q, R>(PhantomData<(Q, R)>);

impl<Q, R, S> Query<S> for With<Q, R>
where
    Q: Query<S>,
    R: Query<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = FetchWith<Q::Fetch<'w>, R::Fetch<'w>>;

    fn check_borrows(checker: &mut BorrowChecker) {
        Q::check_borrows(checker);
    }
}

pub struct Without<Q, R>(PhantomData<(Q, R)>);

impl<Q, R, S> Query<S> for Without<Q, R>
where
    Q: Query<S>,
    R: Query<S>,
    S: ArchetypeSet,
{
    type Fetch<'w> = FetchWithout<Q::Fetch<'w>, R::Fetch<'w>>;

    fn check_borrows(checker: &mut BorrowChecker) {
        Q::check_borrows(checker);
    }
}

// Safety: Before constructing a `FetchIter`, use `BorrowChecker` to ensure that
// the query does not specify borrows that violate Rust's borrowing rules. Also,
// do not allow constructing references to the entity at which the `FetchIter`
// currently points that would violate Rust's borrowing rules.
pub struct FetchIter<'w, 'f, F, S> {
    i: usize,
    fetch: F,
    _phantom: PhantomData<&'w &'f S>,
}

impl<'w, 'f, F, S> FetchIter<'w, 'f, F, S> {
    pub fn new(fetch: F) -> Self {
        Self {
            i: 0,
            fetch,
            _phantom: PhantomData,
        }
    }
}

impl<'w, 'f, F, S> Iterator for FetchIter<'w, 'f, F, S>
where
    F: Fetch<'w, S>,
    S: ArchetypeSet,
{
    type Item = F::Item<'w>;

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

pub struct ArchetypeSetFetchIter<'w, 'f, F, S>
where
    F: Fetch<'w, S>,
    S: ArchetypeSet,
    'w: 'f,
{
    archetype_set_iter: <S::Fetch<'w, F> as ArchetypeSetFetch<'w, S>>::Iter,
    current_fetch_iter: Option<FetchIter<'w, 'f, F, S>>,
}

impl<'w, 'f, F, S> Iterator for ArchetypeSetFetchIter<'w, 'f, F, S>
where
    F: Fetch<'w, S>,
    S: ArchetypeSet,
    'w: 'f,
{
    type Item = <F as Fetch<'w, S>>::Item<'f>;

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

impl<'w, 'f, F, S> ArchetypeSetFetchIter<'w, 'f, F, S>
where
    F: Fetch<'w, S>,
    S: ArchetypeSet,
{
    unsafe fn new(archetype_set: &'w S) -> Self {
        let mut archetype_set_iter = archetype_set.fetch::<F>().iter();

        let current_fetch_iter = archetype_set_iter.next().map(FetchIter::new);

        Self {
            archetype_set_iter,
            current_fetch_iter,
        }
    }
}

pub struct QueryResult<'a, Q, S> {
    archetype_set: &'a mut S,
    _phantom: PhantomData<Q>,
}

impl<'w, Q, S> IntoIterator for QueryResult<'w, Q, S>
where
    Q: Query<S>,
    S: ArchetypeSet,
{
    type Item = <Q::Fetch<'w> as Fetch<'w, S>>::Item<'w>;

    type IntoIter = ArchetypeSetFetchIter<'w, 'w, Q::Fetch<'w>, S>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        Q::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: A `QueryResult` exclusively borrows the `archetype_set: &'a
        // mut S`. Also, `into_iter` consumes the `QueryResult` while
        // maintaining the lifetime `'a`. Thus, it is not possible to construct
        // references to entities in `archetype_set` outside of the returned
        // iterator, thereby satisfying the requirement of `FetchIter`.
        unsafe { ArchetypeSetFetchIter::new(self.archetype_set) }
    }
}

impl<'w, Q, S> QueryResult<'w, Q, S>
where
    Q: Query<S>,
    S: ArchetypeSet,
{
    pub(crate) fn new(archetype_set: &'w mut S) -> Self {
        Self {
            archetype_set,
            _phantom: PhantomData,
        }
    }

    pub fn with<R>(self) -> QueryResult<'w, With<Q, R>, S>
    where
        R: Query<S>,
    {
        QueryResult::new(self.archetype_set)
    }

    pub fn without<R>(self) -> QueryResult<'w, Without<Q, R>, S>
    where
        R: Query<S>,
    {
        QueryResult::new(self.archetype_set)
    }

    pub fn join<J>(self) -> JoinQueryResult<'w, Q, J, S>
    where
        J: Query<S>,
    {
        JoinQueryResult {
            archetype_set: self.archetype_set,
            _phantom: PhantomData,
        }
    }

    pub fn join_stream<J>(self) -> JoinStreamQueryResult<'w, Q, J, S>
    where
        J: Query<S>,
    {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules. Note that `JoinStreamQueryResult` ensures
        // that `Q` and `J` never borrow the same entity simultaneously, so we
        // can get away with checking their borrows separately. In fact, this
        // separation is the whole purpose of `join_stream`.
        Q::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));
        J::check_borrows(&mut BorrowChecker::new(type_name::<J>()));

        // Safety: TODO
        let query_iter = unsafe { ArchetypeSetFetchIter::new(self.archetype_set) };
        let join_fetch = self.archetype_set.fetch();

        JoinStreamQueryResult {
            query_iter,
            join_fetch,
        }
    }
}

pub struct Join<'a, J, S>
where
    J: Fetch<'a, S>,
    S: ArchetypeSet + 'a,
{
    ignore_id: Option<S::EntityId>,
    fetch: S::Fetch<'a, J>,
}

pub struct JoinIter<'a, 'b, J, S, I>
where
    J: Fetch<'a, S>,
    S: ArchetypeSet + 'a,
    'a: 'b,
{
    join: &'b Join<'a, J, S>,
    iter: I,
}

impl<'a, 'b, J, S, I> Iterator for JoinIter<'a, 'b, J, S, I>
where
    J: Fetch<'a, S>,
    S: ArchetypeSet + 'a,
    I: Iterator<Item = S::EntityId>,
    'a: 'b,
{
    type Item = J::Item<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.iter.next()?;

        // Safety: TODO
        unsafe { self.join.fetch.get(id) }
    }
}

impl<'a, J, S> Join<'a, J, S>
where
    J: Fetch<'a, S>,
    S: ArchetypeSet + 'a,
{
    // This has to take an exclusive `self` reference to prevent violating
    // Rust's borrowing rules if `J` contains an exclusive borrow, since `get()`
    // could be called multiple times with the same `id`.
    pub fn get<'b>(&'b mut self, id: S::EntityId) -> Option<J::Item<'b>> {
        if let Some(ignore_id) = self.ignore_id {
            if ignore_id == id {
                // TODO: Consider panicking.
                return None;
            }
        }

        unsafe { self.fetch.get(id) }
    }

    // This has to take an exclusive `self` reference for the same reason as
    // `get()`.
    // FIXME: This does not prevent aliasing.
    pub fn iter<'b, I>(&'b mut self, iter: I) -> JoinIter<'a, 'b, J, S, I>
    where
        'a: 'b,
        I: Iterator<Item = S::EntityId>,
    {
        JoinIter { join: self, iter }
    }
}

pub struct JoinArchetypeSetFetchIter<'w, F, J, S>
where
    F: Fetch<'w, S>,
    J: Fetch<'w, S>,
    S: ArchetypeSet,
{
    query_iter: ArchetypeSetFetchIter<'w, 'w, F, S>,
    join_fetch: S::Fetch<'w, J>,
}

impl<'w, F, J, S> Iterator for JoinArchetypeSetFetchIter<'w, F, J, S>
where
    F: Fetch<'w, S>,
    J: Fetch<'w, S>,
    S: ArchetypeSet,
{
    type Item = (<F as Fetch<'w, S>>::Item<'w>, Join<'w, J, S>);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.query_iter.next()?;
        let join = Join {
            ignore_id: None,
            fetch: self.join_fetch.clone(),
        };

        Some((item, join))
    }
}

pub struct JoinQueryResult<'a, Q, J, S> {
    archetype_set: &'a mut S,
    _phantom: PhantomData<(Q, J)>,
}

impl<'w, Q, J, S> IntoIterator for JoinQueryResult<'w, Q, J, S>
where
    Q: Query<S>,
    J: Query<S>,
    S: ArchetypeSet,
{
    type Item = (
        <Q::Fetch<'w> as Fetch<'w, S>>::Item<'w>,
        Join<'w, J::Fetch<'w>, S>,
    );

    type IntoIter = JoinArchetypeSetFetchIter<'w, Q::Fetch<'w>, J::Fetch<'w>, S>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        Q::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        // Safety: TODO
        let query_iter = unsafe { ArchetypeSetFetchIter::new(self.archetype_set) };
        let join_fetch = self.archetype_set.fetch();

        JoinArchetypeSetFetchIter {
            query_iter,
            join_fetch,
        }
    }
}

pub struct JoinStreamQueryResult<'w, Q, J, S>
where
    Q: Query<S>,
    J: Query<S>,
    S: ArchetypeSet,
{
    query_iter: ArchetypeSetFetchIter<'w, 'w, (FetchEntityId<S::EntityId>, Q::Fetch<'w>), S>,
    join_fetch: S::Fetch<'w, J::Fetch<'w>>,
}

impl<'w, Q, J, S> JoinStreamQueryResult<'w, Q, J, S>
where
    Q: Query<S>,
    J: Query<S>,
    S: ArchetypeSet,
{
    pub fn fetch_next(
        &'w mut self,
    ) -> Option<(
        <Q::Fetch<'w> as Fetch<S>>::Item<'w>,
        Join<'w, J::Fetch<'w>, S>,
    )> {
        let Some((id, item)) = self.query_iter.next() else {
            return None;
        };

        let join = Join {
            ignore_id: Some(id),
            fetch: self.join_fetch.clone(),
        };

        Some((item, join))
    }
}
