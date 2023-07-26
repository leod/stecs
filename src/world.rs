use crate::{
    entity::EntityVariant,
    query::{fetch::Fetch, QueryResult},
    Entity, EntityId, Query,
};

pub trait WorldFetch<D: WorldData>: Clone {
    type Fetch: Fetch;
    type Iter: Iterator<Item = Self::Fetch>;

    unsafe fn get<'f>(
        &self,
        id: <D::Entity as Entity>::Id,
    ) -> Option<<Self::Fetch as Fetch>::Item<'f>>
    where
        Self: 'f;

    fn iter(&mut self) -> Self::Iter;
}

pub trait WorldData: Sized + 'static {
    type Entity: EntityVariant<Self::Entity>;

    type Fetch<'w, F: Fetch + 'w>: WorldFetch<Self, Fetch = F>;

    fn spawn<E>(&mut self, entity: E) -> EntityId<E>
    where
        E: EntityVariant<Self::Entity>;

    fn despawn<E>(&mut self, id: EntityId<E>) -> Option<Self::Entity>
    where
        E: EntityVariant<Self::Entity>;

    fn query<Q: Query<Self>>(&mut self) -> QueryResult<Q, Self> {
        QueryResult::new(self)
    }

    fn entity<E>(&self, id: EntityId<E>) -> Option<E::Ref<'_>>
    where
        E: EntityVariant<Self::Entity>;

    #[doc(hidden)]
    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w;
}

pub type World<E> = <E as Entity>::Data;
