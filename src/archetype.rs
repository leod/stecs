use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use thunderdome::Arena;

use crate::{
    column::Column,
    entity::{Columns, ContainsEntity},
    EntityId,
};

pub struct EntityKey<E>(pub thunderdome::Index, PhantomData<E>);

impl<E> EntityKey<E> {
    #[doc(hidden)]
    pub fn new_unchecked(id: thunderdome::Index) -> Self {
        Self(id, PhantomData)
    }
}

impl<E> Clone for EntityKey<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for EntityKey<E> {}

impl<E> Debug for EntityKey<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EntityKey").field(&self.0).finish()
    }
}

impl<E> PartialEq for EntityKey<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone)]
pub struct Archetype<T: Columns> {
    indices: Arena<usize>,
    ids: Column<thunderdome::Index>,
    columns: T,
}

impl<T: Columns> Archetype<T> {
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

    pub fn columns(&self) -> &T {
        &self.columns
    }

    pub fn spawn(&mut self, entity: T::Entity) -> EntityId<T::Entity> {
        let index = self.ids.len();
        let id = EntityKey::new_unchecked(self.indices.insert(index));

        self.ids.push(id.0);
        self.columns.push(entity);

        EntityId::new(id)
    }

    pub fn despawn<EOuter>(&mut self, id: EntityId<T::Entity, EOuter>) -> Option<T::Entity>
    where
        EOuter: ContainsEntity<T::Entity>,
    {
        let index = self.indices.remove(id.get().0)?;

        self.ids.remove(index);

        if let Some(last) = self.ids.last() {
            self.indices[*last] = self.ids.len() - 1;
        }

        Some(self.columns.remove(index))
    }

    /*
        pub fn get(&mut self, id: EntityId<T>) -> Option<EntityRef<T::Entity>> {
            let index = *self.indices.get(id.get().0)?;

            debug_assert!(index < self.ids.len());

            let fetch = <T::Ref<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

            // Safety: TODO
            Some(unsafe { fetch.get(index) })
        }

        pub fn iter(&self) -> impl Iterator<Item = (EntityId<T>, Option<EntityRef<T::Entity>>)> + '_ {
            // Safety: TODO
            let fetch = <T::Ref<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

            self.ids
                .as_slice()
                .iter()
                .map(|id| EntityId::new_unchecked(*id))
                .zip(FetchIter::new(fetch))
        }

        pub fn values(&self) -> impl Iterator<Item = T::Ref<'_>> + '_ {
            // Safety: TODO
            let fetch = <T::Ref<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

            FetchIter::new(fetch)
        }

        pub fn get_mut(&mut self, id: EntityId<T>) -> Option<T::RefMut<'_>> {
            let index = *self.indices.get(id.0)?;

            debug_assert!(index < self.ids.len());

            let fetch = <T::RefMut<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

            // Safety: TODO
            Some(unsafe { fetch.get(index) })
        }

        pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId<T>, T::RefMut<'_>)> + '_ {
            // Safety: TODO
            let fetch = <T::RefMut<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

            self.ids
                .as_slice()
                .iter()
                .map(|id| EntityId::new_unchecked(*id))
                .zip(FetchIter::new(fetch))
        }

        pub fn values_mut(&mut self) -> impl Iterator<Item = T::RefMut<'_>> + '_ {
            // Safety: TODO
            let fetch = <T::RefMut<'_> as EntityBorrow<'_>>::new_fetch(self.ids.len(), &self.columns);

            FetchIter::new(fetch)
        }
    */
}

impl<T: Columns> Default for Archetype<T> {
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

/*
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
*/
