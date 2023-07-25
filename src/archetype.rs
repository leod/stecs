use thunderdome::Arena;

use crate::{
    column::Column,
    entity::{Columns, EntityBorrow},
    query::{fetch::Fetch, iter::FetchIter},
    Entity, EntityId,
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

// TODO: impl<E: Entity> IntoIterator for Archetype<E>
// TODO: impl<'a, E: Entity> IntoIterator for &'a mut Archetype<E>
// TODO: impl<'a, E: Entity> IntoIterator for &'a Archetype<E>

impl<E: Entity> Default for Archetype<E> {
    fn default() -> Self {
        Self {
            indices: Default::default(),
            ids: Default::default(),
            columns: Default::default(),
        }
    }
}
