pub mod borrow_checker;
pub mod fetch;
pub mod iter;
pub mod join;
pub mod nest;

use std::{any::type_name, marker::PhantomData};

use crate::{
    column::{ColumnRawParts, ColumnRawPartsMut},
    entity::EntityVariant,
    world::WorldFetch,
    Component, Entity, EntityId, SecondaryQuery, SecondaryQueryShared, SecondaryWorld, WorldData,
};

use self::{
    borrow_checker::BorrowChecker,
    fetch::{Fetch, OptionFetch, UnitFetch, WithFetch, WithoutFetch},
    join::JoinQueryBorrow,
    nest::NestQueryBorrow,
};

pub trait Query {
    type Fetch<'w>: Fetch + 'w;
}

pub type QueryItem<'w, Q> = <<Q as Query>::Fetch<'w> as Fetch>::Item<'w>;

pub trait QueryShared: Query {}

impl<'q, C: Component> Query for &'q C {
    type Fetch<'w> = ColumnRawParts<C>;
}

impl<'q, C: Component> QueryShared for &'q C {}

impl<'q, C: Component> Query for &'q mut C {
    type Fetch<'w> = ColumnRawPartsMut<C>;
}

impl<E: Entity> Query for EntityId<E> {
    type Fetch<'w> = E::FetchId<'w>;
}

impl<E: Entity> QueryShared for EntityId<E> {}

macro_rules! tuple_impl {
    () => {
        impl Query for () {
            type Fetch<'w> = UnitFetch;
        }
    };
    ($($name: ident),*) => {
        impl<$($name: Query,)*> Query for ($($name,)*) {
            type Fetch<'w> = ($($name::Fetch<'w>,)*);
        }

        impl<$($name: QueryShared,)*> QueryShared for ($($name,)*) {
        }
    };
}

smaller_tuples_too!(
    tuple_impl, F0, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15
);

pub struct With<Q, R>(PhantomData<(Q, R)>);

impl<Q, R> Query for With<Q, R>
where
    Q: Query,
    R: Query,
{
    type Fetch<'w> = WithFetch<Q::Fetch<'w>, R::Fetch<'w>>;
}

impl<Q, R> QueryShared for With<Q, R>
where
    Q: QueryShared,
    R: Query,
{
}

pub struct Without<Q, R>(PhantomData<(Q, R)>);

impl<Q, R> Query for Without<Q, R>
where
    Q: Query,
    R: Query,
{
    type Fetch<'w> = WithoutFetch<Q::Fetch<'w>, R::Fetch<'w>>;
}

impl<Q, R> QueryShared for Without<Q, R>
where
    Q: QueryShared,
    R: Query,
{
}

#[derive(Debug, Clone, Copy)]
pub enum Or<L, R> {
    Left(L),
    Right(R),
    Both(L, R),
}

impl<L, R> Query for Or<L, R>
where
    L: Query,
    R: Query,
{
    type Fetch<'w> = Or<L::Fetch<'w>, R::Fetch<'w>>;
}

impl<L, R> QueryShared for Or<L, R>
where
    L: QueryShared,
    R: QueryShared,
{
}

impl<Q> Query for Option<Q>
where
    Q: Query,
{
    type Fetch<'w> = OptionFetch<Q::Fetch<'w>>;
}

impl<Q> QueryShared for Option<Q> where Q: QueryShared {}

pub struct QueryBorrow<'w, Q, D> {
    data: &'w D,
    _phantom: PhantomData<Q>,
}

impl<'w, Q, D> QueryBorrow<'w, Q, D>
where
    Q: Query,
    D: WorldData,
{
    pub(crate) fn new(data: &'w D) -> Self {
        Self {
            data,
            _phantom: PhantomData,
        }
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

    pub fn get_mut<'a, E>(
        &'a mut self,
        id: EntityId<E>,
    ) -> Option<<Q::Fetch<'a> as Fetch>::Item<'a>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        // TODO: Cache?

        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'a> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        let world_fetch = self.data.fetch::<Q::Fetch<'a>>();

        // Safety: TODO
        unsafe { world_fetch.get(id.get()) }
    }
}

impl<'w, Q, D> QueryBorrow<'w, Q, D>
where
    Q: QueryShared,
    D: WorldData,
{
    pub fn get<'a, E>(&'a self, id: EntityId<E>) -> Option<<Q::Fetch<'a> as Fetch>::Item<'a>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        let id = id.to_outer();

        // TODO: Cache?

        // Safety: Check that the query does not specify borrows that violate
        // Rust's borrowing rules.
        <Q::Fetch<'a> as Fetch>::check_borrows(&mut BorrowChecker::new(type_name::<Q>()));

        let world_fetch = self.data.fetch::<Q::Fetch<'a>>();

        // Safety: TODO
        unsafe { world_fetch.get(id.get()) }
    }
}

pub struct ExclusiveQueryBorrow<'w, Q, D>(QueryBorrow<'w, Q, D>);

impl<'w, Q, D> ExclusiveQueryBorrow<'w, Q, D>
where
    Q: Query,
    D: WorldData,
{
    pub(crate) fn new(data: &'w mut D) -> Self {
        Self(QueryBorrow::new(data))
    }

    pub fn with<R>(self) -> ExclusiveQueryBorrow<'w, With<Q, R>, D>
    where
        R: Query,
    {
        ExclusiveQueryBorrow(self.0.with::<R>())
    }

    pub fn without<R>(self) -> ExclusiveQueryBorrow<'w, Without<Q, R>, D>
    where
        R: Query,
    {
        ExclusiveQueryBorrow(self.0.without::<R>())
    }

    pub fn nest<R>(self) -> NestQueryBorrow<'w, Q, R, D>
    where
        R: Query,
    {
        NestQueryBorrow {
            data: self.0.data,
            _phantom: PhantomData,
        }
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

    pub fn get_mut<'a, E>(
        &'a mut self,
        id: EntityId<E>,
    ) -> Option<<Q::Fetch<'a> as Fetch>::Item<'a>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        self.0.get_mut(id)
    }
}

impl<'w, Q, D> ExclusiveQueryBorrow<'w, Q, D>
where
    Q: QueryShared,
    D: WorldData,
{
    pub fn get<'a, E>(&'a self, id: EntityId<E>) -> Option<<Q::Fetch<'a> as Fetch>::Item<'a>>
    where
        'w: 'a,
        E: EntityVariant<D::Entity>,
    {
        self.0.get(id)
    }
}
