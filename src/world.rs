use crate::{
    entity::EntityVariant,
    query::{fetch::Fetch, QueryBorrow, QueryShared},
    Entity, EntityId, EntityRef, EntityRefMut, Query,
};

// TODO: This should probably be generic in `Fetch` rather than `WorldData`, but
// this works for now.
pub trait WorldFetch<'w, D: WorldData>: Clone {
    type Fetch: Fetch;
    type Iter: Iterator<Item = Self::Fetch>;

    unsafe fn get<'f>(
        &self,
        id: <D::Entity as Entity>::Id,
    ) -> Option<<Self::Fetch as Fetch>::Item<'f>>;

    fn iter(&mut self) -> Self::Iter;

    fn filter_by_outer<DOuter: WorldData>(&mut self) {}
}

pub trait WorldData: Default + Sized + 'static {
    type Entity: EntityVariant<Self::Entity>;

    type Fetch<'w, F: Fetch + 'w>: WorldFetch<'w, Self, Fetch = F>;

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

    // TODO: Introduce `QueryShared` and `query`.

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

    #[doc(hidden)]
    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w;
}

pub type World<E> = <E as Entity>::WorldData;
