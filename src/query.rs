pub mod fetch;
pub mod iter;
pub mod join;
pub mod nest;
pub mod nest2;

use std::{any::TypeId, marker::PhantomData};

use crate::{
    column::{ColumnRawParts, ColumnRawPartsMut},
    entity::EntityVariant,
    world::WorldFetch,
    Component, Entity, EntityId, SecondaryQuery, SecondaryQueryShared, SecondaryWorld, WorldData,
};

use self::{
    fetch::{Fetch, OptionFetch, UnitFetch, WithFetch, WithoutFetch},
    join::JoinQueryBorrow,
    nest::NestQueryBorrow,
};

// This is unafe because `for_each_borrow` must match `Fetch`.
pub unsafe trait Query {
    type Fetch<'w>: Fetch + 'w;

    fn for_each_borrow(f: impl FnMut(TypeId, bool));
}

pub type QueryItem<'w, 'a, Q> = <<Q as Query>::Fetch<'w> as Fetch>::Item<'a>;

// This is unsafe because it must not have any exclusive borrows.
pub unsafe trait QueryShared: Query {}

unsafe impl<'q, C: Component> Query for &'q C {
    type Fetch<'w> = ColumnRawParts<C>;

    fn for_each_borrow(mut f: impl FnMut(TypeId, bool)) {
        f(TypeId::of::<C>(), false);
    }
}

unsafe impl<'q, C: Component> QueryShared for &'q C {}

unsafe impl<'q, C: Component> Query for &'q mut C {
    type Fetch<'w> = ColumnRawPartsMut<C>;

    fn for_each_borrow(mut f: impl FnMut(TypeId, bool)) {
        f(TypeId::of::<C>(), true);
    }
}

unsafe impl<E: Entity> Query for EntityId<E> {
    type Fetch<'w> = E::FetchId<'w>;

    fn for_each_borrow(_: impl FnMut(TypeId, bool)) {}
}

unsafe impl<E: Entity> QueryShared for EntityId<E> {}

macro_rules! tuple_impl {
    () => {
        unsafe impl Query for () {
            type Fetch<'w> = UnitFetch;

            fn for_each_borrow(_: impl FnMut(TypeId, bool)) {}
        }
    };
    ($($name: ident),*) => {
        unsafe impl<$($name: Query,)*> Query for ($($name,)*) {
            type Fetch<'w> = ($($name::Fetch<'w>,)*);

            #[allow(unused_mut)]
            fn for_each_borrow(mut f: impl FnMut(TypeId, bool)) {
                $($name::for_each_borrow(&mut f);)*
            }
        }

        unsafe impl<$($name: QueryShared,)*> QueryShared for ($($name,)*) {
        }
    };
}

smaller_tuples_too!(
    tuple_impl, F0, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15
);

pub struct With<Q, R>(PhantomData<(Q, R)>);

unsafe impl<Q, R> Query for With<Q, R>
where
    Q: Query,
    R: Query,
{
    type Fetch<'w> = WithFetch<Q::Fetch<'w>, R::Fetch<'w>>;

    fn for_each_borrow(f: impl FnMut(TypeId, bool)) {
        Q::for_each_borrow(f);
    }
}

unsafe impl<Q, R> QueryShared for With<Q, R>
where
    Q: QueryShared,
    R: Query,
{
}

pub struct Without<Q, R>(PhantomData<(Q, R)>);

unsafe impl<Q, R> Query for Without<Q, R>
where
    Q: Query,
    R: Query,
{
    type Fetch<'w> = WithoutFetch<Q::Fetch<'w>, R::Fetch<'w>>;

    fn for_each_borrow(f: impl FnMut(TypeId, bool)) {
        Q::for_each_borrow(f);
    }
}

unsafe impl<Q, R> QueryShared for Without<Q, R>
where
    Q: QueryShared,
    R: Query,
{
}

// Inspired by `hecs`.
#[derive(Debug, Clone, Copy)]
pub enum Or<L, R> {
    Left(L),
    Right(R),
    Both(L, R),
}

impl<L, R> Or<L, R> {
    pub fn new(left: Option<L>, right: Option<R>) -> Option<Self> {
        match (left, right) {
            (None, None) => None,
            (Some(left), None) => Some(Or::Left(left)),
            (None, Some(right)) => Some(Or::Right(right)),
            (Some(left), Some(right)) => Some(Or::Both(left, right)),
        }
    }

    pub fn split(self) -> (Option<L>, Option<R>) {
        use Or::*;

        match self {
            Left(left) => (Some(left), None),
            Right(right) => (None, Some(right)),
            Both(left, right) => (Some(left), Some(right)),
        }
    }

    pub fn left(self) -> Option<L> {
        use Or::*;

        match self {
            Left(left) => Some(left),
            Right(_) => None,
            Both(left, _) => Some(left),
        }
    }

    pub fn right(self) -> Option<R> {
        use Or::*;

        match self {
            Left(_) => None,
            Right(right) => Some(right),
            Both(_, right) => Some(right),
        }
    }

    pub fn map<L1, R1, F, G>(self, f: F, g: G) -> Or<L1, R1>
    where
        F: FnOnce(L) -> L1,
        G: FnOnce(R) -> R1,
    {
        use Or::*;

        match self {
            Left(left) => Left(f(left)),
            Right(right) => Right(g(right)),
            Both(left, right) => Both(f(left), g(right)),
        }
    }

    pub fn as_ref(&self) -> Or<&L, &R> {
        use Or::*;

        match self {
            Left(left) => Left(left),
            Right(right) => Right(right),
            Both(left, right) => Both(left, right),
        }
    }

    pub fn as_mut(&mut self) -> Or<&mut L, &mut R> {
        use Or::*;

        match self {
            Left(left) => Left(left),
            Right(right) => Right(right),
            Both(left, right) => Both(left, right),
        }
    }
}

unsafe impl<L, R> Query for Or<L, R>
where
    L: Query,
    R: Query,
{
    type Fetch<'w> = Or<L::Fetch<'w>, R::Fetch<'w>>;

    fn for_each_borrow(mut f: impl FnMut(TypeId, bool)) {
        L::for_each_borrow(&mut f);
        R::for_each_borrow(&mut f);
    }
}

unsafe impl<L, R> QueryShared for Or<L, R>
where
    L: QueryShared,
    R: QueryShared,
{
}

unsafe impl<Q> Query for Option<Q>
where
    Q: Query,
{
    type Fetch<'w> = OptionFetch<Q::Fetch<'w>>;

    fn for_each_borrow(mut f: impl FnMut(TypeId, bool)) {
        Q::for_each_borrow(&mut f);
    }
}

unsafe impl<Q> QueryShared for Option<Q> where Q: QueryShared {}

pub struct QueryBorrow<'w, Q, D>
where
    Q: Query,
    D: WorldData,
{
    data: &'w D,
    fetch: D::Fetch<'w, Q::Fetch<'w>>,
    _phantom: PhantomData<Q>,
}

impl<'w, Q, D> QueryBorrow<'w, Q, D>
where
    Q: Query,
    D: WorldData,
{
    pub(crate) fn new(data: &'w D) -> Self {
        // Safety: The query must satisfy Rust's borrowing rules.
        assert_borrow::<Q>();

        Self {
            data,
            fetch: data.fetch(),
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.fetch.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn with<R>(self) -> QueryBorrow<'w, With<Q, R>, D>
    where
        R: Query,
    {
        QueryBorrow::new(self.data)
    }

    pub fn without<R>(self) -> QueryBorrow<'w, Without<Q, R>, D>
    where
        R: Query,
    {
        QueryBorrow::new(self.data)
    }

    pub fn join<J>(
        self,
        secondary_world: &'w SecondaryWorld<D::Entity>,
    ) -> JoinQueryBorrow<'w, Q, J, D>
    where
        J: SecondaryQueryShared<D::Entity>,
    {
        JoinQueryBorrow {
            data: self.data,
            secondary_world,
            _phantom: PhantomData,
        }
    }

    pub fn join_mut<J>(
        self,
        secondary_world: &'w SecondaryWorld<D::Entity>,
    ) -> JoinQueryBorrow<'w, Q, J, D>
    where
        J: SecondaryQuery<D::Entity>,
    {
        JoinQueryBorrow {
            data: self.data,
            secondary_world,
            _phantom: PhantomData,
        }
    }

    pub fn get_mut<'a, E>(&'a mut self, id: EntityId<E>) -> Option<QueryItem<'w, 'a, Q>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        // Safety: TODO
        unsafe { self.fetch.get(id.get()) }
    }
}

impl<'w, Q, D> QueryBorrow<'w, Q, D>
where
    Q: QueryShared,
    D: WorldData,
{
    pub fn get<'a, E>(&'a self, id: EntityId<E>) -> Option<QueryItem<'w, 'a, Q>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        // Safety: TODO
        unsafe { self.fetch.get(id.get()) }
    }
}

pub struct QueryMut<'w, Q, D>(QueryBorrow<'w, Q, D>)
where
    Q: Query,
    D: WorldData;

impl<'w, Q, D> QueryMut<'w, Q, D>
where
    Q: Query,
    D: WorldData,
{
    pub(crate) fn new(data: &'w mut D) -> Self {
        Self(QueryBorrow::new(data))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn with<R>(self) -> QueryMut<'w, With<Q, R>, D>
    where
        R: Query,
    {
        QueryMut(self.0.with::<R>())
    }

    pub fn without<R>(self) -> QueryMut<'w, Without<Q, R>, D>
    where
        R: Query,
    {
        QueryMut(self.0.without::<R>())
    }

    pub fn join<J>(
        self,
        secondary_world: &'w SecondaryWorld<D::Entity>,
    ) -> JoinQueryBorrow<'w, Q, J, D>
    where
        J: SecondaryQueryShared<D::Entity>,
    {
        self.0.join(secondary_world)
    }

    pub fn join_mut<J>(
        self,
        secondary_world: &'w mut SecondaryWorld<D::Entity>,
    ) -> JoinQueryBorrow<'w, Q, J, D>
    where
        J: SecondaryQuery<D::Entity>,
    {
        self.0.join_mut(secondary_world)
    }

    pub fn get_mut<'a, E>(&'a mut self, id: EntityId<E>) -> Option<QueryItem<'w, 'a, Q>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        self.0.get_mut(id)
    }

    pub fn nest<J>(self) -> NestQueryBorrow<'w, Q, J, D>
    where
        J: Query,
    {
        NestQueryBorrow::new(self.0.data)
    }
}

impl<'w, Q, D> QueryMut<'w, Q, D>
where
    Q: QueryShared,
    D: WorldData,
{
    pub fn get<'a, E>(&'a self, id: EntityId<E>) -> Option<QueryItem<'w, 'a, Q>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        self.0.get(id)
    }
}

// Adapted from hecs (https://github.com/Ralith/hecs).
pub(crate) fn assert_borrow<Q: Query>() {
    // This looks like an ugly O(n^2) loop, but everything's constant after inlining, so in
    // practice LLVM optimizes it out entirely.
    let mut i = 0;
    Q::for_each_borrow(|a, unique| {
        if unique {
            let mut j = 0;
            Q::for_each_borrow(|b, _| {
                if i != j {
                    core::assert!(a != b, "query violates a unique borrow");
                }
                j += 1;
            })
        }
        i += 1;
    });
}
