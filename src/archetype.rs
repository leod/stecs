use std::{
    any::{type_name, TypeId},
    fmt::{self, Debug},
    hash::{Hash, Hasher},
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
        f.debug_tuple(&format!("EntityKey::<{}>", type_name::<E>()))
            .field(&self.0)
            .finish()
    }
}

impl<E> PartialEq for EntityKey<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<E> Eq for EntityKey<E> {}

impl<E> PartialOrd for EntityKey<E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for EntityKey<E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<E> Hash for EntityKey<E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Clone)]
pub struct Archetype<T: Columns> {
    indices: Arena<usize>,
    ids: Column<thunderdome::Index>,
    columns: T,
}

impl<T: Columns> Archetype<T> {
    fn spawn_impl(&mut self, entity: T::Entity) -> EntityId<T::Entity> {
        let index = self.ids.len();
        let id = EntityKey::new_unchecked(self.indices.insert(index));

        self.ids.push(id.0);
        self.columns.push(entity);

        EntityId::new(id)
    }

    fn despawn_impl(&mut self, id: EntityId<T::Entity>) -> Option<T::Entity> {
        let index = self.indices.remove(id.get().0)?;
        let is_last = index + 1 == self.ids.len();

        self.ids.remove(index);

        if !is_last {
            self.indices[*self.ids.get(index)] = index;
        }

        Some(self.columns.remove(index))
    }

    pub fn get_impl(&mut self, id: EntityId<T::Entity>) -> Option<EntityRef<T::Entity>> {
        let index = *self.indices.get(id.get().0)?;

        debug_assert!(index < self.ids.len());

        let fetch = self.columns.new_fetch(self.ids.len());

        // Safety: TODO
        Some(unsafe { fetch.get(index) })
    }
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

// TODO: impl<T> IntoIterator for Archetype<T>

#[derive(Clone, Copy)]
pub struct ArchetypeWorldFetch<'w, F>(&'w Arena<usize>, Option<F>);

impl<'w, T, F> WorldFetch<'w, Archetype<T>> for ArchetypeWorldFetch<'w, F>
where
    T: Columns,
    F: Fetch + 'w,
{
    type Fetch = F;

    type Iter = option::IntoIter<F>;

    unsafe fn get<'f>(&self, id: EntityKey<T::Entity>) -> Option<F::Item<'f>> {
        self.1
            .and_then(|fetch| self.0.get(id.0).map(|&index| fetch.get(index)))
    }

    fn iter(&mut self) -> Self::Iter {
        self.1.into_iter()
    }

    fn filter_by_outer<DOuter: WorldData>(&mut self) {
        F::filter_by_outer::<DOuter>(&mut self.1)
    }
}

// FIXME: This is a bad hack. There might be a cleaner way with traits.
pub fn adopt_entity_id_unchecked<ESrc, EDst>(id: EntityId<ESrc>) -> EntityId<EDst>
where
    ESrc: Entity,
    EDst: Entity,
{
    // This holds because `Columns::Entity` types are leaf entities, i.e.
    // they do not contain inner entities (other than themselves,
    // trivially).
    assert_eq!(TypeId::of::<ESrc>(), TypeId::of::<EDst>());

    // This is a consequence of the assertion above.
    assert_eq!(TypeId::of::<ESrc::Id>(), TypeId::of::<EDst::Id>());

    // Safety: FIXME and TODO. By the assertion above, we know that the
    // source and destination types are equivalent. Also, `Entity::Id` is
    // `Copy`, so it cannot be `Drop`, and it cannot contain exclusive
    // references. However, it is unclear if these assumptions are strong
    // enough for the call below to be safe.
    let id = id.get();
    let id = unsafe { transmute_copy::<ESrc::Id, EDst::Id>(&id) };

    EntityId::new(id)
}

impl<T: Columns> WorldData for Archetype<T> {
    type Entity = T::Entity;

    type Fetch<'w, F: Fetch + 'w> = ArchetypeWorldFetch<'w, F>;

    fn spawn<E>(&mut self, entity: E) -> EntityId<E>
    where
        E: EntityVariant<Self::Entity>,
    {
        let id = self.spawn_impl(entity.into_outer());

        adopt_entity_id_unchecked(id)
    }

    fn despawn<E>(&mut self, id: EntityId<E>) -> Option<Self::Entity>
    where
        E: EntityVariant<Self::Entity>,
    {
        self.despawn_impl(id.to_outer())
    }

    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w,
    {
        ArchetypeWorldFetch(&self.indices, F::new(&self.ids, &self.columns))
    }
}
