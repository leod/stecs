use crate::{
    entity::ContainsEntity,
    query::{fetch::Fetch, QueryResult},
    Entity, EntityId, Query,
};

pub trait DataFetch<D: Data>: Clone {
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

pub trait Data: Sized + 'static {
    type Entity: Entity;

    type Fetch<'w, F: Fetch + 'w>: DataFetch<Self, Fetch = F>;

    fn spawn<E>(&mut self, entity: E) -> EntityId<E, Self::Entity>
    where
        E: Entity,
        Self::Entity: ContainsEntity<E>;

    fn despawn<E>(&mut self, id: EntityId<E, Self::Entity>) -> Option<Self::Entity>
    where
        E: Entity,
        Self::Entity: ContainsEntity<E>;

    fn get<E>(&self, id: EntityId<E, Self::Entity>) -> Option<E::Ref<'_>>
    where
        E: Entity,
        Self::Entity: ContainsEntity<E>;

    fn query<Q: Query<Self>>(&mut self) -> QueryResult<Q, Self> {
        QueryResult::new(self)
    }

    #[doc(hidden)]
    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w;
}
