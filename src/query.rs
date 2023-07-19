use std::{iter::Zip, marker::PhantomData};

use crate::{Archetype, ArchetypeSet, Component, Entity, InArchetypeSet};

pub trait Query<S: ArchetypeSet> {
    type Iter<E: Entity>: Iterator<Item = Self>;

    fn iter_archetype<E: InArchetypeSet<S>>(archetype: &mut Archetype<E>) -> Option<Self::Iter<E>>;
}

impl<'a, C, S> Query<S> for &'a C
where
    C: Component,
    S: ArchetypeSet,
{
    type Iter<E: Entity> = UnsafeColumnIter<'a, C>;

    fn iter_archetype<E: InArchetypeSet<S>>(archetype: &mut Archetype<E>) -> Option<Self::Iter<E>> {
        let (ptr, len) = archetype.column::<C>()?;

        Some(UnsafeColumnIter {
            ptr,
            len,
            _phantom: PhantomData,
        })
    }
}

impl<'a, C, S> Query<S> for &'a mut C
where
    C: Component,
    S: ArchetypeSet,
{
    type Iter<E: Entity> = UnsafeColumnIterMut<'a, C>;

    fn iter_archetype<E: InArchetypeSet<S>>(archetype: &mut Archetype<E>) -> Option<Self::Iter<E>> {
        let (ptr, len) = archetype.column::<C>()?;

        Some(UnsafeColumnIterMut {
            ptr,
            len,
            _phantom: PhantomData,
        })
    }
}

impl<'a, Q0, Q1, S> Query<S> for (Q0, Q1)
where
    Q0: Query<S>,
    Q1: Query<S>,
    S: ArchetypeSet,
{
    type Iter<E: Entity> = Zip<Q0::Iter<E>, Q1::Iter<E>>;

    fn iter_archetype<E: InArchetypeSet<S>>(archetype: &mut Archetype<E>) -> Option<Self::Iter<E>> {
        Some(Q0::iter_archetype(archetype)?.zip(Q1::iter_archetype(archetype)?))
    }
}

pub struct UnsafeColumnIter<'a, C> {
    ptr: *const C,
    len: usize,
    _phantom: PhantomData<&'a C>,
}

impl<'a, C> Iterator for UnsafeColumnIter<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            let ptr = self.ptr;

            self.ptr = unsafe { self.ptr.add(1) };
            self.len -= 1;

            Some(unsafe { &*ptr })
        }
    }
}

pub struct UnsafeColumnIterMut<'a, C> {
    ptr: *mut C,
    len: usize,
    _phantom: PhantomData<&'a C>,
}

impl<'a, C> Iterator for UnsafeColumnIterMut<'a, C> {
    type Item = &'a mut C;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            let ptr = self.ptr;

            self.ptr = unsafe { self.ptr.add(1) };
            self.len -= 1;

            Some(unsafe { &mut *ptr })
        }
    }
}
