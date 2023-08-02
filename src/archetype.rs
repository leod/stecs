use std::{
    any::{type_name, TypeId},
    fmt::{self, Debug},
    marker::PhantomData,
    mem::transmute_copy,
    option,
};

use derivative::Derivative;
use thunderdome::Arena;

use crate::{
    column::Column,
    entity::{Columns, EntityVariant},
    query::fetch::Fetch,
    world::WorldFetch,
    Entity, EntityId, EntityRef, WorldData,
};

#[derive(Derivative)]
#[derivative(
    Copy(bound = ""),
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = ""),
    Hash(bound = "")
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityKey<E>(
    #[cfg_attr(feature = "serde", serde(with = "serde_index"))] pub thunderdome::Index,
    PhantomData<E>,
);

// FIXME: Figure out the serialization story. By itself,
// serializing/deserializing a `thunderdome::Index` is not meaningful.
#[cfg(feature = "serde")]
mod serde_index {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(index: &thunderdome::Index, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(index.to_bits())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<thunderdome::Index, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u64::deserialize(deserializer)?;
        thunderdome::Index::from_bits(bits)
            .ok_or_else(|| serde::de::Error::custom("Failed to deserialize thunderdome::Index"))
    }
}

impl<E> EntityKey<E> {
    #[doc(hidden)]
    pub fn new_unchecked(id: thunderdome::Index) -> Self {
        Self(id, PhantomData)
    }
}

impl<E> Debug for EntityKey<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(&format!("EntityKey::<{}>", type_name::<E>()))
            .field(&self.0)
            .finish()
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

    fn spawn_at_impl(&mut self, id: EntityId<T::Entity>, entity: T::Entity) {
        self.indices.insert_at(id.get().0, self.ids.len());

        self.ids.push(id.get().0);
        self.columns.push(entity);
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

pub struct ArchetypeWorldFetch<'w, F, T>(&'w Arena<usize>, Option<F>, PhantomData<T>);

impl<'w, F: Copy, T> Clone for ArchetypeWorldFetch<'w, F, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'w, F: Copy, T> Copy for ArchetypeWorldFetch<'w, F, T> {}

impl<'w, F, T> WorldFetch<'w, F> for ArchetypeWorldFetch<'w, F, T>
where
    F: Fetch + 'w,
    T: Columns,
{
    type Data = Archetype<T>;

    type Iter = option::IntoIter<F>;

    unsafe fn get<'f>(&self, id: EntityKey<T::Entity>) -> Option<F::Item<'f>> {
        self.1
            .and_then(|fetch| self.0.get(id.0).map(|&index| fetch.get(index)))
    }

    fn iter(&mut self) -> Self::Iter {
        self.1.into_iter()
    }

    fn len(&self) -> usize {
        if self.1.is_some() {
            self.0.len()
        } else {
            0
        }
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

    type Fetch<'w, F: Fetch + 'w> = ArchetypeWorldFetch<'w, F, T>;

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

    fn spawn_at(
        &mut self,
        id: EntityId<Self::Entity>,
        entity: Self::Entity,
    ) -> Option<Self::Entity> {
        let old = self.despawn(id);

        self.spawn_at_impl(id, entity);

        old
    }

    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w,
    {
        ArchetypeWorldFetch(&self.indices, F::new(&self.ids, &self.columns), PhantomData)
    }
}

// Safety: TODO. This is needed because `T` can contain `RefCell`. However, this
// is thread-safe, because `WorldData` only allows mutation with `&mut self`.
unsafe impl<T: Columns> Send for Archetype<T> {}
unsafe impl<T: Columns> Sync for Archetype<T> {}
