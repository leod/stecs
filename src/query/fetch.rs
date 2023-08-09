use std::{any::TypeId, marker::PhantomData};

use crate::{
    archetype::EntityKey,
    column::{Column, ColumnRawParts, ColumnRawPartsMut},
    entity::{Columns, EntityStruct},
    Component, EntityId,
};

use super::Or;

// TODO: Now that borrow checking is in Query, maybe this no longer needs to be
// unsafe.
pub unsafe trait Fetch: Copy {
    type Item<'a>
    where
        Self: 'a;

    fn new<T: Columns>(ids: &Column<thunderdome::Index>, columns: &T) -> Option<Self>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Fetches the components specified by `Self::Query` for the entity stored
    /// at `index`.
    ///
    /// # Safety
    ///
    /// This is unsafe because it shifts the burden of checking Rust's borrowing
    /// rules to the caller. The caller has to ensure that this method is not
    /// called on an `index` whose components are already borrowed elsewhere (be
    /// it through `self` or not through `self`).
    ///
    /// The method also does not do bounds checking.
    unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
    where
        Self: 'a;
}

unsafe impl<C> Fetch for ColumnRawParts<C>
where
    C: Component,
{
    type Item<'a> = &'a C where Self: 'a;

    fn new<T: Columns>(_: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
        columns.column::<C>().map(|column| column.as_raw_parts())
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
    where
        Self: 'a,
    {
        debug_assert!(index < <Self as Fetch>::len(self));

        unsafe { &*self.ptr.add(index) }
    }
}

unsafe impl<C> Fetch for ColumnRawPartsMut<C>
where
    C: Component,
{
    type Item<'a> = &'a mut C where Self: 'a;

    fn new<T: Columns>(_: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
        columns
            .column::<C>()
            .map(|column| column.as_raw_parts_mut())
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
    where
        Self: 'a,
    {
        debug_assert!(index < <Self as Fetch>::len(self));

        unsafe { &mut *self.ptr.add(index) }
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
    type Item<'a> = EntityId<E>;

    fn new<T: Columns>(ids: &Column<thunderdome::Index>, _: &T) -> Option<Self> {
        if TypeId::of::<T::Entity>() == TypeId::of::<E>() {
            Some(Self(ids.as_raw_parts(), PhantomData))
        } else {
            None
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
    where
        Self: 'a,
    {
        EntityId::new(EntityKey::new_unchecked(*Fetch::get(&self.0, index)))
    }
}

macro_rules! tuple_impl {
    () => {
        #[derive(Copy, Clone)]
        pub struct UnitFetch(usize);

        unsafe impl Fetch for UnitFetch {
            type Item<'a> = ();

            fn new<T: Columns>(ids: &Column<thunderdome::Index>, _: &T) -> Option<Self> {
                Some(Self(ids.len()))
            }

            #[inline]
            fn len(&self) -> usize {
                self.0
            }

            #[inline]
            unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a> {
                debug_assert!(index < self.len());
            }
        }
    };
    ($($name: ident),*) => {
        unsafe impl<$($name: Fetch,)*> Fetch for ($($name,)*) {
            type Item<'a> = ($($name::Item<'a>,)*) where Self: 'a;

            #[allow(non_snake_case, unused)]
            fn new<T: Columns>(ids: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
                let len = ids.len();
                $(let $name = $name::new(ids, columns)?;)*
                $(assert_eq!($name.len(), len);)*

                Some(($($name,)*))
            }

            #[inline]
            fn len(&self) -> usize {
                self.0.len()
            }

            #[inline]
            unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
            where
                Self: 'a,
            {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                ($($name.get(index),)*)
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
    type Item<'a> = F::Item<'a> where Self: 'a;

    fn new<T: Columns>(ids: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
        let fetch = F::new(ids, columns)?;

        R::new(ids, columns)?;

        Some(Self {
            fetch,
            _phantom: PhantomData,
        })
    }

    #[inline]
    fn len(&self) -> usize {
        self.fetch.len()
    }

    #[inline]
    unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
    where
        Self: 'a,
    {
        self.fetch.get(index)
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
    type Item<'a> = F::Item<'a> where Self: 'a;

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

    #[inline]
    fn len(&self) -> usize {
        self.fetch.len()
    }

    #[inline]
    unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
    where
        Self: 'a,
    {
        self.fetch.get(index)
    }
}

unsafe impl<L, R> Fetch for Or<L, R>
where
    L: Fetch,
    R: Fetch,
{
    type Item<'a> = Or<L::Item<'a>, R::Item<'a>> where Self: 'a;

    fn new<T: Columns>(ids: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
        Or::new(L::new(ids, columns), R::new(ids, columns))
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            Or::Left(left) => left.len(),
            Or::Right(right) => right.len(),
            Or::Both(left, _) => left.len(),
        }
    }

    #[inline]
    unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
    where
        Self: 'a,
    {
        match self {
            Or::Left(left) => Or::Left(left.get(index)),
            Or::Right(right) => Or::Right(right.get(index)),
            Or::Both(left, right) => Or::Both(left.get(index), right.get(index)),
        }
    }
}

#[derive(Clone, Copy)]
pub struct OptionFetch<F> {
    fetch: Option<F>,
    len: usize,
}

unsafe impl<F> Fetch for OptionFetch<F>
where
    F: Fetch,
{
    type Item<'a> = Option<F::Item<'a>> where Self: 'a;

    fn new<T: Columns>(ids: &Column<thunderdome::Index>, columns: &T) -> Option<Self> {
        Some(OptionFetch {
            fetch: F::new(ids, columns),
            len: ids.len(),
        })
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
    where
        Self: 'a,
    {
        self.fetch.map_or(None, |fetch| Some(fetch.get(index)))
    }
}
