use std::{
    any::TypeId,
    iter,
    mem::{self, transmute},
    option,
};

use thunderdome::Arena;

use crate::{
    archetype_set::ArchetypeSetFetch,
    column::Column,
    entity::{Columns, EntityBorrow},
    query::{fetch::Fetch, iter::FetchIter},
    ArchetypeSet, Entity, EntityId,
};

#[derive(Clone)]
pub struct Archetype<E: Entity> {
    indices: Arena<usize>,
    ids: Column<thunderdome::Index>,
    columns: E::Columns,
}

impl<E: Entity> Archetype<E> {
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn indices(&self) -> &Arena<usize> {
        &self.indices
    }

    pub fn ids(&self) -> &Column<thunderdome::Index> {
        &self.ids
    }

    pub fn columns(&self) -> &E::Columns {
        &self.columns
    }

    pub fn spawn(&mut self, entity: E) -> EntityId<E> {
        let index = self.ids.len();
        let id = EntityId::new_unchecked(self.indices.insert(index));

        self.ids.push(id.0);
        self.columns.push(entity);

        id
    }

    pub fn despawn(&mut self, id: EntityId<E>) -> Option<E> {
        let index = self.indices.remove(id.0)?;

        self.ids.remove(index);

        if let Some(last) = self.ids.last() {
            self.indices[*last] = self.ids.len() - 1;
        }

        Some(self.columns.remove(index))
    }

    pub fn get(&mut self, id: EntityId<E>) -> Option<E::Borrow<'_>> {
        let index = *self.indices.get(id.0)?;

        debug_assert!(index < self.ids.len());

        let fetch = <E::Borrow<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

        // Safety: TODO
        Some(unsafe { fetch.get(index) })
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId<E>, E::Borrow<'_>)> + '_ {
        // Safety: TODO
        let fetch = <E::Borrow<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

        self.ids
            .as_slice()
            .iter()
            .map(|id| EntityId::new_unchecked(*id))
            .zip(FetchIter::new(fetch))
    }

    pub fn values(&self) -> impl Iterator<Item = E::Borrow<'_>> + '_ {
        // Safety: TODO
        let fetch = <E::Borrow<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

        FetchIter::new(fetch)
    }

    pub fn get_mut(&mut self, id: EntityId<E>) -> Option<E::BorrowMut<'_>> {
        let index = *self.indices.get(id.0)?;

        debug_assert!(index < self.ids.len());

        let fetch =
            <E::BorrowMut<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

        // Safety: TODO
        Some(unsafe { fetch.get(index) })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId<E>, E::BorrowMut<'_>)> + '_ {
        // Safety: TODO
        let fetch =
            <E::BorrowMut<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

        self.ids
            .as_slice()
            .iter()
            .map(|id| EntityId::new_unchecked(*id))
            .zip(FetchIter::new(fetch))
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = E::BorrowMut<'_>> + '_ {
        // Safety: TODO
        let fetch =
            <E::BorrowMut<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

        FetchIter::new(fetch)
    }
}

impl<E: Entity> Default for Archetype<E> {
    fn default() -> Self {
        Self {
            indices: Default::default(),
            ids: Default::default(),
            columns: Default::default(),
        }
    }
}

// TODO: impl<E: Entity> IntoIterator for Archetype<E>
// TODO: impl<'a, E: Entity> IntoIterator for &'a mut Archetype<E>
// TODO: impl<'a, E: Entity> IntoIterator for &'a Archetype<E>

#[derive(Clone, Copy)]
pub struct SingletonFetch<'w, F>(&'w Arena<usize>, Option<F>);

impl<'w, E, F> ArchetypeSetFetch<Archetype<E>> for SingletonFetch<'w, F>
where
    E: Entity,
    F: Fetch,
{
    type Fetch = F;

    type Iter = option::IntoIter<F>;

    unsafe fn get<'f>(&self, id: EntityId<E>) -> Option<F::Item<'f>>
    where
        Self: 'f,
    {
        self.1
            .and_then(|fetch| self.0.get(id.0).map(|&index| fetch.get(index)))
    }

    fn iter(&mut self) -> Self::Iter {
        self.1.into_iter()
    }
}

impl<E: Entity> ArchetypeSet for Archetype<E> {
    type AnyEntityId = EntityId<E>;

    type AnyEntity = E;

    type Fetch<'w, F: Fetch + 'w> = SingletonFetch<'w, F>
    where
        Self: 'w;

    fn spawn(&mut self, entity: E) -> Self::AnyEntityId {
        self.spawn(entity)
    }

    fn despawn(&mut self, id: Self::AnyEntityId) -> Option<Self::AnyEntity> {
        self.despawn(id)
    }

    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w,
    {
        SingletonFetch(&self.indices, F::new(&self.ids, &self.columns))
    }
}
