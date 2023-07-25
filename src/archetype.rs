use thunderdome::Arena;

use crate::{
    column::Column,
    entity::{BorrowEntity, Columns},
    query::fetch::Fetch,
    Entity, EntityKey,
};

#[derive(Clone)]
pub struct Archetype<E: Entity> {
    indices: Arena<usize>,
    untyped_keys: Column<thunderdome::Index>,
    columns: E::Columns,
}

impl<E: Entity> Archetype<E> {
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
}

impl<E: Entity> Default for Archetype<E> {
    fn default() -> Self {
        Self {
            indices: Default::default(),
            untyped_keys: Default::default(),
            columns: Default::default(),
        }
    }
}
