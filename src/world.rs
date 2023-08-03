use std::any::type_name;

use crate::{
    entity::EntityVariant,
    query::{borrow_checker::BorrowChecker, fetch::Fetch, QueryBorrow, QueryShared},
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

pub trait WorldData: Send + Sync + Default + Clone + 'static {
    type Entity: EntityVariant<Self::Entity>;

    type Fetch<'w, F: Fetch + 'w>: WorldFetch<'w, F, Data = Self>;

    fn new() -> Self {
        // TODO: Panic if there is a duplicate entity type anywhere.
        Self::default()
    }

    fn spawn<E>(&mut self, entity: E) -> EntityId<E>
    where
        E: EntityVariant<Self::Entity>;

    fn despawn<E>(&mut self, id: EntityId<E>) -> Option<Self::Entity>
    where
        E: EntityVariant<Self::Entity>;

    fn query<Q: QueryShared>(&self) -> QueryBorrow<Q, Self> {
        QueryBorrow::new(self)
    }

    fn query_mut<Q: Query>(&mut self) -> QueryBorrow<Q, Self> {
        QueryBorrow::new(self)
    }

    fn queries<Q: MultiQueryShared>(&self) -> Q::QueryBorrows<'_, Self> {
        unsafe { Q::new(self) }
    }

    fn queries_mut<Q: MultiQuery>(&mut self) -> Q::QueryBorrows<'_, Self> {
        unsafe { Q::new(self) }
    }

    fn get<Q: QueryShared>(
        &self,
        id: EntityId<Self::Entity>,
    ) -> Option<<Q::Fetch<'_> as Fetch>::Item<'_>> {
        let fetch = self.fetch::<<Q as Query>::Fetch<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    fn get_mut<Q: Query>(
        &mut self,
        id: EntityId<Self::Entity>,
    ) -> Option<<Q::Fetch<'_> as Fetch>::Item<'_>> {
        let fetch = self.fetch::<<Q as Query>::Fetch<'_>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    fn entity<'w, E>(&'w self, id: EntityId<E>) -> Option<EntityRef<'w, E>>
    where
        E: EntityVariant<Self::Entity>,

        // TODO: Can we put the bound below on `Entity` somehow?
        <E::Ref<'w> as Query>::Fetch<'w>: Fetch<Item<'w> = EntityRef<'w, E>>,
    {
        let id = id.to_outer();
        let fetch = self.fetch::<<E::Ref<'w> as Query>::Fetch<'w>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

    fn entity_mut<'w, E>(&'w mut self, id: EntityId<E>) -> Option<EntityRefMut<'w, E>>
    where
        E: EntityVariant<Self::Entity>,

        // TODO: Can we put the bound below on `Entity` somehow?
        <E::RefMut<'w> as Query>::Fetch<'w>: Fetch<Item<'w> = EntityRefMut<'w, E>>,
    {
        let id = id.to_outer();
        let fetch = self.fetch::<<E::RefMut<'w> as Query>::Fetch<'w>>();

        // Safety: TODO
        unsafe { fetch.get(id.get()) }
    }

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

pub type World<E> = <E as Entity>::WorldData;

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
