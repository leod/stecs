use std::{iter::Zip, marker::PhantomData};

use crate::{Archetype, ArchetypeSet, Component, Entity, InArchetypeSet};

// TODO: `Query` probably does not need a lifetime.
pub trait Query<'a, S: ArchetypeSet> {
    type Iter<E: Entity>: Iterator<Item = Self>
    where
        E: 'a;

    fn iter_archetype<E: InArchetypeSet<S>>(archetype: &'a Archetype<E>) -> Option<Self::Iter<E>>
    where
        E: 'a;
}

impl<'a, C, S> Query<'a, S> for &'a C
where
    C: Component,
    S: ArchetypeSet,
{
    type Iter<E: Entity> = UnsafeColumnIter<'a, C> where E: 'a;

    fn iter_archetype<E: InArchetypeSet<S>>(archetype: &Archetype<E>) -> Option<Self::Iter<E>>
    where
        E: 'a,
    {
        let (ptr, len) = archetype.column::<C>()?;

        Some(UnsafeColumnIter {
            ptr,
            len,
            _phantom: PhantomData,
        })
    }
}

impl<'a, C, S> Query<'a, S> for &'a mut C
where
    C: Component,
    S: ArchetypeSet,
{
    type Iter<E: Entity> = UnsafeColumnIterMut<'a, C> where E: 'a;

    fn iter_archetype<E: InArchetypeSet<S>>(archetype: &Archetype<E>) -> Option<Self::Iter<E>>
    where
        E: 'a,
    {
        let (ptr, len) = archetype.column::<C>()?;

        Some(UnsafeColumnIterMut {
            ptr,
            len,
            _phantom: PhantomData,
        })
    }
}

impl<'a, Q0, Q1, S> Query<'a, S> for (Q0, Q1)
where
    Q0: Query<'a, S>,
    Q1: Query<'a, S>,
    S: ArchetypeSet,
{
    type Iter<E: Entity> = Zip<Q0::Iter<E>, Q1::Iter<E>> where E: 'a;

    fn iter_archetype<E: InArchetypeSet<S>>(archetype: &'a Archetype<E>) -> Option<Self::Iter<E>> {
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
