use std::marker::PhantomData;

use crate::{arena, Archetype, Component, World, WorldArchetype};

// FIXME: Figure out safety!

pub trait Getter<'a, W, A>
where
    W: WorldArchetype<A>,
    A: Archetype,
{
    type Output;

    unsafe fn get(&self, id: W::EntityId, entity: &'a mut A) -> Self::Output;
}

pub struct GetterIter<'a, W, A, G>
where
    W: WorldArchetype<A>,
    A: Archetype,
    G: Getter<'a, W, A>,
{
    iter: arena::iter::IterMut<'a, A>,
    getter: Option<G>,
    _phantom: PhantomData<(W, A)>,
}

impl<'a, W, A, G> GetterIter<'a, W, A, G>
where
    W: WorldArchetype<A>,
    A: Archetype,
    G: Getter<'a, W, A>,
{
    pub fn new(iter: arena::iter::IterMut<'a, A>, getter: Option<G>) -> Self {
        GetterIter {
            iter,
            getter,
            _phantom: PhantomData,
        }
    }
}

impl<'a, W, A, G> Iterator for GetterIter<'a, W, A, G>
where
    W: WorldArchetype<A>,
    A: Archetype,
    G: Getter<'a, W, A>,
{
    type Item = G::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let getter = self.getter.as_ref()?;
        let (index, entity) = self.iter.next()?;
        let id = W::id(index);

        // FIXME: Figure out safety.
        Some(unsafe { getter.get(id, entity) })
    }
}

pub trait Query<'a, W>
where
    W: World,
{
    type Getter<A>: Getter<'a, W, A, Output = Self>
    where
        W: WorldArchetype<A>,
        A: Archetype + 'a;

    fn getter<A: Archetype + 'a>() -> Option<Self::Getter<A>>
    where
        W: WorldArchetype<A>;
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

pub struct EntityIdGetter;

impl<'a, W, A> Getter<'a, W, A> for EntityIdGetter
where
    W: WorldArchetype<A>,
    A: Archetype,
{
    type Output = W::EntityId;

    unsafe fn get(&self, id: W::EntityId, _: &'a mut A) -> Self::Output {
        id
    }
}

pub struct ComponentGetter<A, C> {
    offset: usize,
    _phantom: PhantomData<(A, C)>,
}

impl<'a, W, A, C> Getter<'a, W, A> for ComponentGetter<A, &'a C>
where
    W: WorldArchetype<A>,
    A: Archetype,
    C: Component,
{
    type Output = &'a C;

    // FIXME: Figure out if this can even be done safely.
    unsafe fn get(&self, _: W::EntityId, entity: &'a mut A) -> Self::Output {
        let entity = entity as *const A as *const ();
        let component = entity.add(self.offset) as *const C;

        &*component
    }
}

impl<'a, W, A, C> Getter<'a, W, A> for ComponentGetter<A, &'a mut C>
where
    W: WorldArchetype<A>,
    A: Archetype,
    C: Component,
{
    type Output = &'a mut C;

    // FIXME: Figure out if this can even be done safely.
    unsafe fn get(&self, _: W::EntityId, entity: &'a mut A) -> Self::Output {
        let entity = entity as *mut A as *mut ();
        let component = entity.add(self.offset) as *mut C;

        &mut *component
    }
}

impl<'a, W, C> Query<'a, W> for &'a C
where
    W: World,
    C: Component,
{
    type Getter<A> = ComponentGetter<A, &'a C>
    where
        W: WorldArchetype<A>,
        A: Archetype + 'a;

    fn getter<A>() -> Option<Self::Getter<A>>
    where
        W: WorldArchetype<A>,
        A: Archetype + 'a,
    {
        let offset = A::offset_of::<C>()?;

        Some(ComponentGetter {
            offset,
            _phantom: PhantomData,
        })
    }
}

impl<'a, W, C> Query<'a, W> for &'a mut C
where
    W: World,
    C: Component,
{
    type Getter<A> = ComponentGetter<A, &'a mut C>
    where
        W: WorldArchetype<A>,
        A: Archetype + 'a;

    fn getter<A>() -> Option<Self::Getter<A>>
    where
        W: WorldArchetype<A>,
        A: Archetype + 'a,
    {
        let offset = A::offset_of::<C>()?;

        Some(ComponentGetter {
            offset,
            _phantom: PhantomData,
        })
    }
}

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
