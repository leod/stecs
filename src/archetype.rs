use thunderdome::Arena;

use crate::{
    column::Column,
    entity::{BorrowEntity, Columns},
    query::{fetch::Fetch, iter::FetchIter},
    Entity, EntityKey,
};

#[derive(Clone)]
pub struct Archetype<E: Entity> {
    indices: Arena<usize>,
    untyped_keys: Column<thunderdome::Index>,
    columns: E::Columns,
}

impl<E: Entity> Archetype<E> {
    pub fn len(&self) -> usize {
        self.untyped_keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn indices(&self) -> &Arena<usize> {
        &self.indices
    }

    pub fn untyped_keys(&self) -> &Column<thunderdome::Index> {
        &self.untyped_keys
    }

    pub fn columns(&self) -> &E::Columns {
        &self.columns
    }

    pub fn spawn(&mut self, entity: E) -> EntityKey<E> {
        let index = self.untyped_keys.len();
        let key = EntityKey::new_unchecked(self.indices.insert(index));

        self.untyped_keys.push(key.0);
        self.columns.push(entity);

        key
    }

    pub fn despawn(&mut self, key: EntityKey<E>) -> Option<E> {
        let index = self.indices.remove(key.0)?;

        self.untyped_keys.remove(index);

        if let Some(last) = self.untyped_keys.last() {
            self.indices[*last] = self.untyped_keys.len() - 1;
        }

        Some(self.columns.remove(index))
    }

    pub fn get(&mut self, key: EntityKey<E>) -> Option<E::Borrow<'_>> {
        let index = *self.indices.get(key.0)?;

        debug_assert!(index < self.untyped_keys.len());

        let fetch =
            <E::Borrow<'_> as BorrowEntity<'_>>::new_fetch(self.untyped_keys.len(), &self.columns);

        // Safety: TODO
        Some(unsafe { fetch.get(index) })
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityKey<E>, E::Borrow<'_>)> + '_ {
        // Safety: TODO
        let fetch =
            <E::Borrow<'_> as BorrowEntity<'_>>::new_fetch(self.untyped_keys.len(), &self.columns);

        self.untyped_keys
            .as_slice()
            .iter()
            .map(|key| EntityKey::new_unchecked(*key))
            .zip(FetchIter::new(fetch))
    }

    pub fn values(&self) -> impl Iterator<Item = E::Borrow<'_>> + '_ {
        // Safety: TODO
        let fetch =
            <E::Borrow<'_> as BorrowEntity<'_>>::new_fetch(self.untyped_keys.len(), &self.columns);

        FetchIter::new(fetch)
    }

    pub fn get_mut(&mut self, key: EntityKey<E>) -> Option<E::BorrowMut<'_>> {
        let index = *self.indices.get(key.0)?;

        debug_assert!(index < self.untyped_keys.len());

        let fetch = <E::BorrowMut<'_> as BorrowEntity<'_>>::new_fetch(
            self.untyped_keys.len(),
            &self.columns,
        );

        // Safety: TODO
        Some(unsafe { fetch.get(index) })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityKey<E>, E::BorrowMut<'_>)> + '_ {
        // Safety: TODO
        let fetch = <E::BorrowMut<'_> as BorrowEntity<'_>>::new_fetch(
            self.untyped_keys.len(),
            &self.columns,
        );

        self.untyped_keys
            .as_slice()
            .iter()
            .map(|key| EntityKey::new_unchecked(*key))
            .zip(FetchIter::new(fetch))
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = E::BorrowMut<'_>> + '_ {
        // Safety: TODO
        let fetch = <E::BorrowMut<'_> as BorrowEntity<'_>>::new_fetch(
            self.untyped_keys.len(),
            &self.columns,
        );

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
            untyped_keys: Default::default(),
            columns: Default::default(),
        }
    }
}
