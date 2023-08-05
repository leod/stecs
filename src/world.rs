use std::{any::type_name, fmt::Debug};

use derivative::Derivative;

use crate::{
    entity::EntityVariant,
    query::{borrow_checker::BorrowChecker, fetch::Fetch, QueryBorrow, QueryItem, QueryShared},
    Entity, EntityId, EntityRef, EntityRefMut, Query,
};

pub trait WorldFetch<'w, F: Fetch>: Clone {
    type Data: WorldData;
    type Iter: Iterator<Item = F>;

    unsafe fn get<'a>(
        &self,
        id: <<Self::Data as WorldData>::Entity as Entity>::Id,
    ) -> Option<F::Item<'a>>;

    fn iter(&mut self) -> Self::Iter;

    fn len(&self) -> usize;
}

pub trait WorldData: Default + Clone + 'static {
    type Entity: EntityVariant<Self::Entity>;

    type Fetch<'w, F: Fetch + 'w>: WorldFetch<'w, F, Data = Self>;

    fn spawn<E>(&mut self, entity: E) -> EntityId<E>
    where
        E: EntityVariant<Self::Entity>;

    fn despawn<E>(&mut self, id: EntityId<E>) -> Option<Self::Entity>
    where
        E: EntityVariant<Self::Entity>;

    fn spawn_at(
        &mut self,
        id: EntityId<Self::Entity>,
        entity: Self::Entity,
    ) -> Option<Self::Entity>;

    #[doc(hidden)]
    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w;
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = "E::WorldData: Clone"),
    Debug(bound = "E::WorldData: Debug"),
    Default(bound = "")
)]
pub struct World<E: Entity>(E::WorldData);

impl<E: Entity> World<E> {
    pub fn new() -> Self {
        // TODO: Panic if there is a duplicate entity type anywhere.
        Self::default()
    }

    pub fn spawn<F>(&mut self, entity: F) -> EntityId<F>
    where
        F: EntityVariant<E>,
    {
        self.0.spawn(entity)
    }

    pub fn despawn<F>(&mut self, id: EntityId<F>) -> Option<E>
    where
        F: EntityVariant<E>,
    {
        self.0.despawn(id)
    }

    pub fn query<Q: QueryShared>(&self) -> QueryBorrow<Q, E::WorldData> {
        QueryBorrow::new(&self.0)
    }

    pub fn query_mut<Q: Query>(&mut self) -> QueryBorrow<Q, E::WorldData> {
        QueryBorrow::new(&self.0)
    }

    pub fn queries<Q: MultiQueryShared>(&self) -> Q::QueryBorrows<'_, E::WorldData> {
        unsafe { Q::new(&self.0) }
    }

    pub fn queries_mut<Q: MultiQuery>(&mut self) -> Q::QueryBorrows<'_, E::WorldData> {
        unsafe { Q::new(&self.0) }
    }

    pub fn get<Q: QueryShared>(&self, id: EntityId<E>) -> Option<QueryItem<Q>> {
        let fetch = self.0.fetch::<<Q as Query>::Fetch<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    pub fn get_mut<Q: Query>(&mut self, id: EntityId<E>) -> Option<QueryItem<Q>> {
        let fetch = self.0.fetch::<<Q as Query>::Fetch<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    pub fn entity<F>(&self, id: EntityId<F>) -> Option<EntityRef<F>>
    where
        F: EntityVariant<E>,
    {
        let id = id.to_outer();
        let fetch = self.0.fetch::<F::Fetch<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    pub fn entity_mut<F>(&mut self, id: EntityId<F>) -> Option<EntityRefMut<F>>
    where
        F: EntityVariant<E>,
    {
        let id = id.to_outer();
        let fetch = self.0.fetch::<F::FetchMut<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    pub fn spawn_at(&mut self, id: EntityId<E>, entity: E) -> Option<E> {
        self.0.spawn_at(id, entity)
    }
}

pub trait MultiQuery {
    type QueryBorrows<'w, D: WorldData>;

    unsafe fn new<D: WorldData>(world: &D) -> Self::QueryBorrows<'_, D>;
}

pub trait MultiQueryShared: MultiQuery {}

macro_rules! tuple_impl {
    ($($name: ident),*) => {
        #[allow(unused)]
        impl<$($name: Query,)*> MultiQuery for ($($name,)*) {
            type QueryBorrows<'w, D: WorldData> = ($(QueryBorrow<'w, $name, D>,)*);

            #[allow(clippy::needless_lifetimes, clippy::unused_unit)]
            unsafe fn new<'w, D: WorldData>(world: &'w D) -> Self::QueryBorrows<'w, D> {
                let mut checker = BorrowChecker::new(type_name::<Self>());

                // Safety: Check that the query does not specify borrows that violate
                // Rust's borrowing rules.
                $(<$name::Fetch<'w> as Fetch>::check_borrows(&mut checker);)*

                ($(QueryBorrow::<$name, _>::new(world),)*)
            }
        }

        impl<$($name: QueryShared,)*> MultiQueryShared for ($($name,)*) {}
    };
}

smaller_tuples_too!(
    tuple_impl, F0, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15
);

// For proc macros.
#[doc(hidden)]
pub type EntityWorldData<E> = <E as Entity>::WorldData;

// For proc macros.
#[doc(hidden)]
pub type EntityWorldFetch<'w, E, F> = <EntityWorldData<E> as WorldData>::Fetch<'w, F>;

// For proc macros.
#[doc(hidden)]
pub type EntityWorldFetchIter<'w, E, F> = <EntityWorldFetch<'w, E, F> as WorldFetch<'w, F>>::Iter;
