use std::{
    any::{type_name, TypeId},
    collections::HashSet,
    marker::PhantomData,
};

use crate::{arena, Archetype, Component, World, WorldArchetype};

// FIXME: Figure out safety!

#[derive(Debug, Clone)]
pub struct BorrowChecker {
    query: &'static str,
    borrows: HashSet<TypeId>,
    mut_borrows: HashSet<TypeId>,
}

impl BorrowChecker {
    fn new(query: &'static str) -> Self {
        BorrowChecker {
            query,
            borrows: Default::default(),
            mut_borrows: Default::default(),
        }
    }

    fn borrow<C: 'static>(&mut self) {
        let type_id = TypeId::of::<C>();

        if self.mut_borrows.contains(&type_id) {
            self.panic_exclusive_and_shared::<C>();
        }

        self.borrows.insert(type_id);
    }

    fn borrow_mut<C: 'static>(&mut self) {
        let type_id = TypeId::of::<C>();

        if self.borrows.contains(&type_id) {
            self.panic_exclusive_and_shared::<C>();
        }

        if self.mut_borrows.contains(&type_id) {
            self.panic_exclusive_and_exclusive::<C>();
        }

        self.mut_borrows.insert(type_id);
    }

    fn panic_exclusive_and_shared<C>(&self) -> ! {
        panic!(
            "Query `{}` has an exclusive and a shared reference to component `{}`",
            self.query,
            type_name::<C>(),
        );
    }

    fn panic_exclusive_and_exclusive<C>(&self) -> ! {
        panic!(
            "Query `{}` has multiple exclusive references to component `{}`",
            self.query,
            type_name::<C>(),
        );
    }
}

pub trait Getter<'a, W, A>
where
    W: World,
    A: WorldArchetype<W>,
{
    type Output;

    unsafe fn get(&self, id: W::EntityId, entity: *mut A) -> Self::Output;
}

pub struct GetterIter<'a, W, A, G>
where
    W: World,
    A: WorldArchetype<W>,
    G: Getter<'a, W, A>,
{
    iter: arena::iter::IterMut<'a, A>,
    getter: Option<G>,
    _phantom: PhantomData<(W, A)>,
}

impl<'a, W, A, G> GetterIter<'a, W, A, G>
where
    W: World,
    A: WorldArchetype<W>,
    G: Getter<'a, W, A>,
{
    pub fn new<Q>(iter: arena::iter::IterMut<'a, A>) -> Self
    where
        Q: Query<'a, W, Getter<A> = G>,
    {
        // SAFETY: Check that the components in the query satisfy Rust's
        // borrowing rules.
        let mut borrow_checker = BorrowChecker::new(type_name::<Q>());
        Q::check_borrows(&mut borrow_checker);

        let getter = Q::getter::<A>();

        GetterIter {
            iter,
            getter,
            _phantom: PhantomData,
        }
    }
}

impl<'a, W, A, G> Iterator for GetterIter<'a, W, A, G>
where
    W: World,
    A: WorldArchetype<W>,
    G: Getter<'a, W, A>,
{
    type Item = G::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let getter = self.getter.as_ref()?;
        let (index, entity) = self.iter.next()?;
        let id = A::id(index);

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
        A: 'a + WorldArchetype<W>;

    fn check_borrows(checker: &mut BorrowChecker);

    fn getter<A>() -> Option<Self::Getter<A>>
    where
        A: 'a + WorldArchetype<W>;
}

pub struct EntityIdGetter;

impl<'a, W, A> Getter<'a, W, A> for EntityIdGetter
where
    W: World,
    A: WorldArchetype<W>,
{
    type Output = W::EntityId;

    unsafe fn get(&self, id: W::EntityId, _: *mut A) -> Self::Output {
        id
    }
}

pub struct ComponentGetter<A, C> {
    offset: usize,
    _phantom: PhantomData<(A, C)>,
}

impl<'a, W, A, C> Getter<'a, W, A> for ComponentGetter<A, &'a C>
where
    W: World,
    A: WorldArchetype<W>,
    C: Component,
{
    type Output = &'a C;

    // FIXME: Figure out if this can even be done safely.
    unsafe fn get(&self, _: W::EntityId, entity: *mut A) -> Self::Output {
        let entity = entity as *const A as *const u8;
        let component = entity.add(self.offset) as *const C;

        &*component
    }
}

impl<'a, W, A, C> Getter<'a, W, A> for ComponentGetter<A, &'a mut C>
where
    W: World,
    A: WorldArchetype<W>,
    C: Component,
{
    type Output = &'a mut C;

    // FIXME: Figure out if this can even be done safely.
    unsafe fn get(&self, _: W::EntityId, entity: *mut A) -> Self::Output {
        let entity = entity as *mut A as *mut u8;
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
        A: 'a + WorldArchetype<W>;

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow::<C>();
    }

    fn getter<A>() -> Option<Self::Getter<A>>
    where
        A: 'a + WorldArchetype<W>,
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
        A: 'a + WorldArchetype<W>;

    fn check_borrows(checker: &mut BorrowChecker) {
        checker.borrow_mut::<C>();
    }

    fn getter<A>() -> Option<Self::Getter<A>>
    where
        A: WorldArchetype<W> + 'a,
    {
        let offset = A::offset_of::<C>()?;

        Some(ComponentGetter {
            offset,
            _phantom: PhantomData,
        })
    }
}

pub struct PairGetter<G0, G1>(G0, G1);

impl<'a, W, A, G0, G1> Getter<'a, W, A> for PairGetter<G0, G1>
where
    W: World,
    A: WorldArchetype<W>,
    G0: Getter<'a, W, A>,
    G1: Getter<'a, W, A>,
{
    type Output = (G0::Output, G1::Output);

    unsafe fn get(&self, id: W::EntityId, entity: *mut A) -> Self::Output {
        (self.0.get(id, entity), self.1.get(id, entity))
    }
}

impl<'a, W, Q0, Q1> Query<'a, W> for (Q0, Q1)
where
    W: World,
    Q0: Query<'a, W>,
    Q1: Query<'a, W>,
{
    type Getter<A> = PairGetter<Q0::Getter<A>, Q1::Getter<A>>
    where
        A: 'a + WorldArchetype<W>;

    fn check_borrows(checkers: &mut BorrowChecker) {
        Q0::check_borrows(checkers);
        Q1::check_borrows(checkers);
    }

    fn getter<A>() -> Option<Self::Getter<A>>
    where
        A: 'a + WorldArchetype<W>,
    {
        let g0 = Q0::getter::<A>()?;
        let g1 = Q1::getter::<A>()?;

        Some(PairGetter(g0, g1))
    }
}

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
