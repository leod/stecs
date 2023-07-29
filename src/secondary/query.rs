use crate::{query::borrow_checker::BorrowChecker, Component, Entity, EntityId, SecondaryWorld};

use super::column::SecondaryColumn;

pub trait SecondaryFetch<'w, E: Entity>: Copy + 'w {
    type Item<'f>
    where
        Self: 'f;

    fn new(world: &'w SecondaryWorld<E>) -> Option<Self>;

    /// # Safety
    ///
    /// TODO
    unsafe fn get<'f>(&self, id: EntityId<E>) -> Option<Self::Item<'f>>
    where
        Self: 'f;

    #[doc(hidden)]
    fn check_borrows(checker: &mut BorrowChecker);
}

pub trait SecondaryQuery<E: Entity> {
    type Fetch<'w>: SecondaryFetch<'w, E>;
}

pub trait SecondaryQueryShared<E: Entity>: SecondaryQuery<E> {}

pub struct ComponentFetch<'w, E: Entity, C>(&'w SecondaryColumn<E, C>);

impl<'w, E: Entity, C> Clone for ComponentFetch<'w, E, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'w, E: Entity, C> Copy for ComponentFetch<'w, E, C> {}

impl<'w, E: Entity, C: Component> SecondaryFetch<'w, E> for ComponentFetch<'w, E, C> {
    type Item<'f> = &'f C
    where
        Self: 'f;

    fn new(world: &'w SecondaryWorld<E>) -> Option<Self> {
        world.column::<C>().map(ComponentFetch)
    }

    unsafe fn get<'f>(&self, id: EntityId<E>) -> Option<Self::Item<'f>>
    where
        Self: 'f,
    {
        self.0.get(id).map(|cell| unsafe {
            let ptr = cell.get();

            &*ptr
        })
    }

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow::<C>();
    }
}

impl<'q, E: Entity, C: Component> SecondaryQuery<E> for &'q C {
    type Fetch<'w> = ComponentFetch<'w, E, C>;
}

impl<'q, E: Entity, C: Component> SecondaryQueryShared<E> for &'q C {}

pub struct ComponentMutFetch<'w, E: Entity, C>(&'w SecondaryColumn<E, C>);

impl<'w, E: Entity, C> Clone for ComponentMutFetch<'w, E, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'w, E: Entity, C> Copy for ComponentMutFetch<'w, E, C> {}

impl<'w, E: Entity, C: Component> SecondaryFetch<'w, E> for ComponentMutFetch<'w, E, C> {
    type Item<'f> = &'f mut C
    where
        Self: 'f;

    fn new(world: &'w SecondaryWorld<E>) -> Option<Self> {
        world.column::<C>().map(ComponentMutFetch)
    }

    unsafe fn get<'f>(&self, id: EntityId<E>) -> Option<Self::Item<'f>>
    where
        Self: 'f,
    {
        self.0.get(id).map(|cell| unsafe {
            let ptr = cell.get();

            &mut *ptr
        })
    }

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow_mut::<C>();
    }
}

impl<'q, E: Entity, C: Component> SecondaryQuery<E> for &'q mut C {
    type Fetch<'w> = ComponentMutFetch<'w, E, C>;
}

impl<'q, E: Entity, C: Component> SecondaryQueryShared<E> for &'q mut C {}

macro_rules! tuple_impl {
    ($($name: ident),*) => {
        impl<'w, E: Entity, $($name: SecondaryFetch<'w, E>,)*> SecondaryFetch<'w, E>
        for ($($name,)*) {
            type Item<'f> = ($($name::Item<'f>,)*)
            where
                Self: 'f;

            #[allow(unused)]
            fn new(world: &'w SecondaryWorld<E>) -> Option<Self> {
                Some(($($name::new(world)?,)*))
            }

            /// # Safety
            ///
            /// TODO
            #[allow(unused)]
            unsafe fn get<'f>(&self, id: EntityId<E>) -> Option<Self::Item<'f>>
            where
                Self: 'f,
            {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                Some(($($name.get(id)?,)*))
            }

            #[allow(unused)]
            fn check_borrows(checker: &mut BorrowChecker) {
                $($name::check_borrows(checker);)*
            }
        }

        impl<E: Entity, $($name: SecondaryQuery<E>,)*> SecondaryQuery<E> for ($($name,)*) {
            type Fetch<'w> = ($($name::Fetch<'w>,)*);
        }

        impl<E: Entity, $($name: SecondaryQueryShared<E>,)*> SecondaryQueryShared<E>
        for ($($name,)*) {
        }
    };
}

smaller_tuples_too!(
    tuple_impl, F0, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15
);

// TODO: With/Without
