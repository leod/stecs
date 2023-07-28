use std::{any::TypeId, marker::PhantomData};

use crate::{
    archetype::EntityKey,
    column::{Column, ColumnRawParts, ColumnRawPartsMut},
    entity::{Columns, EntityStruct},
    Component, WorldData,
};

use super::borrow_checker::BorrowChecker;

// TODO: unsafe maybe not needed.
pub unsafe trait Fetch: Copy {
    type Item<'f>
    where
        Self: 'f;

    fn new<T: Columns>(ids: &Column<thunderdome::Index>, columns: &T) -> Option<Self>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

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

    // TODO: This is no longer needed, since we have decided to disallow having
    // the same entity type multiple times via nesting.
    #[doc(hidden)]
    fn filter_by_outer<DOuter: WorldData>(_: &mut Option<Self>) {}
}

unsafe impl<C> Fetch for ColumnRawParts<C>
where
    C: Component,
{
    type Item<'f> = &'f C where Self: 'f;

    fn new<T: Columns>(_: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
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

    fn new<T: Columns>(_: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
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

pub struct EntityKeyFetch<E>(ColumnRawParts<thunderdome::Index>, PhantomData<E>);

impl<E> Clone for EntityKeyFetch<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for EntityKeyFetch<E> {}

unsafe impl<E> Fetch for EntityKeyFetch<E>
where
    E: EntityStruct<Id = EntityKey<E>>,
{
    type Item<'f> = EntityKey<E>;

    fn new<T: Columns>(ids: &Column<thunderdome::Index>, _: &T) -> Option<Self> {
        if TypeId::of::<T::Entity>() == TypeId::of::<E>() {
            Some(Self(ids.as_raw_parts(), PhantomData))
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        EntityKey::new_unchecked(*Fetch::get(&self.0, index))
    }

    fn check_borrows(_: &mut BorrowChecker) {}
}

macro_rules! tuple_impl {
    () => {
        #[derive(Copy, Clone)]
        pub struct UnitFetch(usize);

        unsafe impl Fetch for UnitFetch {
            type Item<'f> = ();

            fn new<T: Columns>(ids: &Column<thunderdome::Index>, _: &T) -> Option<Self> {
                Some(Self(ids.len()))
            }

            fn len(&self) -> usize {
                self.0
            }

            unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f> {
                assert!(index < self.len());

                ()
            }

            fn check_borrows(_: &mut BorrowChecker) {}
        }
    };
    ($($name: ident),*) => {
        unsafe impl<$($name: Fetch,)*> Fetch for ($($name,)*) {
            type Item<'f> = ($($name::Item<'f>,)*) where Self: 'f;

            #[allow(non_snake_case, unused)]
            fn new<T: Columns>(ids: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
                let len = None;
                $(
                    let $name = $name::new(ids, columns)?;

                    if let Some(len) = len {
                        assert_eq!($name.len(), len);
                    }
                    let len = Some($name.len());
                )*

                Some(($($name,)*))
            }

            fn len(&self) -> usize {
                self.0.len()
            }

            unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
            where
                Self: 'f,
            {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                ($($name.get(index),)*)
            }

            fn check_borrows(checker: &mut BorrowChecker) {
                $($name::check_borrows(checker);)*
            }
        }
    };
}

smaller_tuples_too!(
    tuple_impl, F0, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15
);

#[derive(Clone, Copy)]
pub struct WithFetch<F, R> {
    fetch: F,
    _phantom: PhantomData<R>,
}

unsafe impl<F, R> Fetch for WithFetch<F, R>
where
    F: Fetch,
    R: Fetch,
{
    type Item<'f> = F::Item<'f> where Self: 'f;

    fn new<T: Columns>(ids: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
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
pub struct WithoutFetch<F, R> {
    fetch: F,
    _phantom: PhantomData<R>,
}

unsafe impl<F, R> Fetch for WithoutFetch<F, R>
where
    F: Fetch,
    R: Fetch,
{
    type Item<'f> = F::Item<'f> where Self: 'f;

    fn new<T: Columns>(ids: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
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
