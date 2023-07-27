use std::{
    any::TypeId,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::transmute_copy,
    option,
};

use thunderdome::Arena;

use crate::{
    column::Column,
    entity::{Columns, EntityVariant},
    query::fetch::Fetch,
    world::WorldFetch,
    Entity, EntityId, EntityRef, WorldData,
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
    fn len(&self) -> usize {
        self.ids.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn indices(&self) -> &Arena<usize> {
        &self.indices
    }

    fn ids(&self) -> &Column<thunderdome::Index> {
        &self.ids
    }

    fn columns(&self) -> &T {
        &self.columns
    }

    fn spawn_impl(&mut self, entity: T::Entity) -> EntityId<T::Entity> {
        let index = self.ids.len();
        let id = EntityKey::new_unchecked(self.indices.insert(index));

        self.ids.push(id.0);
        self.columns.push(entity);

        EntityId::new_unchecked(id)
    }

    fn despawn_impl(&mut self, id: EntityId<T::Entity>) -> Option<T::Entity> {
        let index = self.indices.remove(id.get().0)?;

        self.ids.remove(index);

        if let Some(last) = self.ids.last() {
            self.indices[*last] = self.ids.len() - 1;
        }

        Some(self.columns.remove(index))
    }

    pub fn get_impl(&mut self, id: EntityId<T::Entity>) -> Option<EntityRef<T::Entity>> {
        let index = *self.indices.get(id.get().0)?;

        debug_assert!(index < self.ids.len());

        let fetch = self.columns.new_fetch(self.ids.len());

        // Safety: TODO
        Some(EntityRef(unsafe { fetch.get(index) }))
    }

    /*

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
pub struct ArchetypeWorldFetch<'w, F>(&'w Arena<usize>, Option<F>);

impl<'w, T, F> WorldFetch<Archetype<T>> for ArchetypeWorldFetch<'w, F>
where
    T: Columns,
    F: Fetch,
{
    type Fetch = F;

    type Iter = option::IntoIter<F>;

    unsafe fn get<'f>(&self, id: EntityKey<T::Entity>) -> Option<F::Item<'f>>
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

impl<T: Columns> WorldData for Archetype<T> {
    type Entity = T::Entity;

    type Fetch<'w, F: Fetch + 'w> = ArchetypeWorldFetch<'w, F>;

    fn spawn<E>(&mut self, entity: E) -> EntityId<E>
    where
        E: EntityVariant<Self::Entity>,
    {
        // This holds because `Columns::Entity` types are leaf entities, i.e.
        // they do not contain inner entities (other than themselves,
        // trivially).
        assert_eq!(TypeId::of::<T::Entity>(), TypeId::of::<E>());

        // This is a consequence of the assertion above.
        assert_eq!(
            TypeId::of::<<T::Entity as Entity>::Id>(),
            TypeId::of::<E::Id>()
        );

        let id = self.spawn(entity.into_outer()).get();

        // Safety: FIXME and TODO. By the assertion above, we know that the
        // source and destination types are equivalent. Also, `Entity::Id` is
        // `Copy`, so it cannot be `Drop`, and it cannot contain exclusive
        // references. However, it is unclear if these assumptions are strong
        // enough for the call below to be safe.
        let id = unsafe { transmute_copy::<<T::Entity as Entity>::Id, E::Id>(&id) };

        EntityId::new_unchecked(id)
    }

    fn despawn<E>(&mut self, id: EntityId<E>) -> Option<Self::Entity>
    where
        E: EntityVariant<Self::Entity>,
    {
        self.despawn_impl(id.to_outer())
    }

    fn entity<E>(&self, id: EntityId<E>) -> Option<E::Ref<'_>>
    where
        E: EntityVariant<Self::Entity>,
    {
        let id = id.to_outer();
        let index = self.indices.get(id.get().0);

        todo!()
    }

    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w,
    {
        ArchetypeWorldFetch(&self.indices, F::new(&self.ids, &self.columns))
    }
}
