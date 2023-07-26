use std::{fmt::Debug, marker::PhantomData};

use crate::{
    query::{fetch::Fetch, QueryResult},
    Entity, EntityId, Query,
};

pub trait ArchetypeSetFetch<S: ArchetypeSet> {
    type Fetch: Fetch;
    type Iter: Iterator<Item = Self::Fetch>;

    unsafe fn get<'f>(&self, id: S::AnyEntityId) -> Option<<Self::Fetch as Fetch>::Item<'f>>
    where
        Self: 'f;

    fn iter(&mut self) -> Self::Iter;
}

pub trait ArchetypeSet: Default + Sized {
    type AnyEntityId: Copy + Debug + PartialEq + 'static;

    type AnyEntity;

    type Fetch<'w, F: Fetch + 'w>: ArchetypeSetFetch<Self, Fetch = F> + Clone + 'w
    where
        Self: 'w;

    fn spawn(&mut self, entity: Self::AnyEntity) -> Self::AnyEntityId;

    fn despawn(&mut self, id: Self::AnyEntityId) -> Option<Self::AnyEntity>;

    fn query<Q: Query<Self>>(&mut self) -> QueryResult<Q, Self> {
        QueryResult::new(self)
    }

    #[doc(hidden)]
    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w;
}

pub type AnyEntityId<S> = <S as ArchetypeSet>::AnyEntityId;

/*
pub struct EmbeddedArchetypeSet<SInner, SOuter>
where
    SInner: ArchetypeSet,
    SOuter: ArchetypeSet,
{
    pub inner: SInner,
    pub embed_entity_id: fn(SInner::AnyEntityId) -> SOuter::AnyEntityId,
    pub _phantom: PhantomData<SOuter>,
}

impl<SInner, SOuter> EmbeddedArchetypeSet<SInner, SOuter>
where
    SInner: ArchetypeSet,
    SOuter: ArchetypeSet,
{
    pub fn spawn(&mut self, entity: SInner::AnyEntity) -> SOuter::AnyEntityId {
        (self.embed_entity_id)(self.inner.entity(entity))
    }
}
*/
