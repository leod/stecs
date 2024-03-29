use std::fmt::Debug;

use derivative::Derivative;

use crate::{
    entity::EntityVariant,
    query::{assert_borrow, fetch::Fetch, QueryBorrow, QueryItem, QueryMut, QueryShared},
    Entity, EntityRef, EntityRefMut, Id, Query,
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

pub trait WorldData: Default + 'static {
    type Entity: EntityVariant<Self::Entity>;

    type Fetch<'w, F: Fetch + 'w>: WorldFetch<'w, F, Data = Self>;

    fn spawn<E>(&mut self, entity: E) -> Id<E>
    where
        E: EntityVariant<Self::Entity>;

    fn despawn<E>(&mut self, id: Id<E>) -> Option<Self::Entity>
    where
        E: EntityVariant<Self::Entity>;

    fn spawn_at(&mut self, id: Id<Self::Entity>, entity: Self::Entity) -> Option<Self::Entity>;

    fn contains(&self, id: Id<Self::Entity>) -> bool;

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
        // TODO: Panic if there is a duplicate component type anywhere.
        Self::default()
    }

    pub fn spawn<F>(&mut self, entity: F) -> Id<F>
    where
        F: EntityVariant<E>,
    {
        self.0.spawn(entity)
    }

    pub fn despawn<F>(&mut self, id: Id<F>) -> Option<E>
    where
        F: EntityVariant<E>,
    {
        self.0.despawn(id)
    }

    pub fn query<Q: QueryShared>(&self) -> QueryBorrow<Q, E::WorldData> {
        QueryBorrow::new(&self.0)
    }

    pub fn query_mut<Q: Query>(&mut self) -> QueryMut<Q, E::WorldData> {
        QueryMut::new(&mut self.0)
    }

    pub fn queries<Q: MultiQueryShared>(&self) -> Q::QueryBorrows<'_, E::WorldData> {
        unsafe { Q::new(&self.0) }
    }

    pub fn queries_mut<Q: MultiQuery>(&mut self) -> Q::QueryBorrows<'_, E::WorldData> {
        unsafe { Q::new(&self.0) }
    }

    pub fn get<Q: QueryShared>(&self, id: Id<E>) -> Option<QueryItem<Q>> {
        let fetch = self.0.fetch::<<Q as Query>::Fetch<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    pub fn get_mut<Q: Query>(&mut self, id: Id<E>) -> Option<QueryItem<Q>> {
        let fetch = self.0.fetch::<<Q as Query>::Fetch<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    pub fn entity<F>(&self, id: Id<F>) -> Option<EntityRef<F>>
    where
        F: EntityVariant<E>,
    {
        let id = id.to_outer();
        let fetch = self.0.fetch::<F::Fetch<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    pub fn entity_mut<F>(&mut self, id: Id<F>) -> Option<EntityRefMut<F>>
    where
        F: EntityVariant<E>,
    {
        let id = id.to_outer();
        let fetch = self.0.fetch::<F::FetchMut<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    pub fn spawn_at(&mut self, id: Id<E>, entity: E) -> Option<E> {
        self.0.spawn_at(id, entity)
    }

    pub fn contains<F>(&self, id: Id<F>) -> bool
    where
        F: EntityVariant<E>,
    {
        self.0.contains(id.to_outer())
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
                // Safety: Check that the query does not specify borrows that violate
                // Rust's borrowing rules.
                $(assert_borrow::<$name>();)*

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
