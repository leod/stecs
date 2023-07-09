use std::{marker::PhantomData, mem::transmute};

use crate::{arena, Archetype, Component};

// FIXME: Figure out safety!

pub trait Getter<'a, A> {
    type Output;

    unsafe fn get(&self, index: arena::Index, entity: &'a mut A) -> Self::Output;
}

pub struct GetterIter<'a, A, G>
where
    G: Getter<'a, A>,
{
    iter: arena::iter::IterMut<'a, A>,
    getter: Option<G>,
    _phantom: PhantomData<A>,
}

impl<'a, A, G> GetterIter<'a, A, G>
where
    G: Getter<'a, A>,
{
    pub fn new(iter: arena::iter::IterMut<'a, A>, getter: Option<G>) -> Self {
        GetterIter {
            iter,
            getter,
            _phantom: PhantomData,
        }
    }
}

impl<'a, A, G> Iterator for GetterIter<'a, A, G>
where
    G: Getter<'a, A>,
{
    type Item = G::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let getter = self.getter.as_ref()?;
        let (index, entity) = self.iter.next()?;

        // FIXME: Figure out safety.
        Some(unsafe { getter.get(index, entity) })
    }
}

pub trait Query<'a> {
    type Getter<A: Archetype + 'a>: Getter<'a, A, Output = Self>;

    fn getter<A: Archetype + 'a>() -> Option<Self::Getter<A>>;
}

macro_rules! zip_type {
    () => { Empty<()> };

    ($lt:lifetime, $name:ty) => { <$name as Query>::Iter<$lt> };

    ($lt:lifetime, $name1:ty, $name2:ty) => {
        Zip<zip_type!($lt, $name1), zip_type!($lt, $name2)>
    };

    ($lt:lifetime, $name:ty, $($rest:ty),*) => {
        Zip<zip_type!($lt, $name), zip_type!($lt, $($rest),*)>
    };
}

pub struct ComponentAccessor<A, C> {
    offset: usize,
    _phantom: PhantomData<(A, C)>,
}

impl<'a, A, C> Getter<'a, A> for ComponentAccessor<A, &'a C>
where
    A: Archetype,
    C: Component,
{
    type Output = &'a C;

    // FIXME: Figure out if this can even be done safely.
    unsafe fn get(&self, _: arena::Index, entity: &'a mut A) -> Self::Output {
        let entity = entity as *const A as *const ();
        let component = entity.add(self.offset) as *const C;

        &*component
    }
}

impl<'a, C: Component> Query<'a> for &'a C {
    type Getter<A: Archetype + 'a> = ComponentAccessor<A, &'a C>;

    fn getter<A: Archetype + 'a>() -> Option<Self::Getter<A>> {
        let offset = A::offset_of::<C>()?;

        Some(ComponentAccessor {
            offset,
            _phantom: PhantomData,
        })
    }
}

/*impl<'a, C: Component> Query<'a> for &'a C {
    type Iter = ColumnValues<'a, C>;

    fn query<A: Archetype>(storage: &'a Storage<A>) -> Option<Self::Iter> {
        storage
            .column()
            .map(move |column| ColumnValues(column.iter()))
    }
}*/

/*
impl<'a, D: Query, E: Query> Query for (&'a D, &'a E) {
    type Iter<'b> = zip_type!('b, &'a D, &'a E)
    where
        Self: 'b;

    fn query<'b, A: Archetype>(storage: &'b Storage<A>) -> Option<Self::Iter<'b>>
    where
        Self: 'b,
    {
        todo!()
    }

*/

/*
macro_rules! tuple_impl {
    ($($name:ident),*) => {
        #[allow(unused_parens)]
        impl<$($name: Query),*> Query for ($($name,)*) {
            type Iter<'a> = zip_type!($($name),*) where Self: 'a;

            fn query<'a, A: Archetype>(storage: &'a Storage<A>) -> Option<Self::Iter<'a>> {
                $(
                    let Some($name) = storage.column::<A>() else { return None; };
                )*

                None
            }
        }
    };
}

smaller_tuples_too!(tuple_impl, F);
*/
