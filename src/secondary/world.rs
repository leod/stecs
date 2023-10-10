use std::{any::TypeId, marker::PhantomData};

use hashbrown::{HashMap, HashSet};

use crate::{
    entity::EntityVariant, query::fetch::Fetch, Component, Entity, EntityId, EntityRef, Query,
    World,
};

use super::column::{AnySecondaryColumn, SecondaryColumn};

// TODO: Joining with the secondary world is currently pretty expensive, since
// it is one hash map lookup for each entity yielded from the primary query. We
// could try to improve this situation by having one hibitset per primary
// archetype in the secondary world.

pub struct SecondaryWorld<E: Entity> {
    ids: HashSet<EntityId<E>>,
    columns: HashMap<TypeId, Box<dyn AnySecondaryColumn<E>>>,
    _phantom: PhantomData<E>,
}

impl<E: Entity> Default for SecondaryWorld<E> {
    fn default() -> Self {
        Self {
            ids: Default::default(),
            columns: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<E: Entity> SecondaryWorld<E> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn column<C: Component>(&self) -> Option<&SecondaryColumn<E, C>> {
        self.columns
            .get(&TypeId::of::<C>())
            .and_then(|column| column.downcast_ref())
    }

    pub fn spawn<B: ComponentBundle<E>>(&mut self, id: EntityId<E>, components: B) -> bool {
        if !self.ids.insert(id) {
            return false;
        }

        self.insert(id, components);

        true
    }

    pub fn despawn(&mut self, id: EntityId<E>) -> bool {
        if !self.ids.remove(&id) {
            return false;
        }

        for column in self.columns.values_mut() {
            column.remove(id);
        }

        true
    }

    pub fn insert<B: ComponentBundle<E>>(&mut self, id: EntityId<E>, components: B) {
        components.insert_entity(self, id);
    }

    // TODO: Allow removing components.

    pub fn synchronize<'w>(
        &'w mut self,
        world: &'w World<E>,
        new_entity: impl Fn(&mut Self, EntityId<E>, E::Borrow<'_>),
    ) where
        // TODO: Can we put the bound below on `Entity` somehow?
        <E::Borrow<'w> as Query>::Fetch<'w>: Fetch<Item<'w> = EntityRef<'w, E>>,
        E: EntityVariant<E>,
    {
        // FIXME: This is too inefficient for something per-frame. We need a
        // better data structure for `SecondaryWorld` so that we do not have a
        // linear number of hash map lookups in `synchronize`.

        for (id, entity) in world.query::<(EntityId<E>, EntityRef<E>)>() {
            if !self.ids.contains(&id) {
                new_entity(self, id, entity);
            }
        }

        let remove_ids: Vec<_> = self
            .ids
            .iter()
            .copied()
            .filter(|id| world.entity(*id).is_none())
            .collect();

        for id in remove_ids {
            self.despawn(id);
        }
    }
}

pub trait ComponentBundle<E: Entity> {
    fn insert_entity(self, world: &mut SecondaryWorld<E>, id: EntityId<E>);
}

macro_rules! tuple_impl {
    ($($name: ident),*) => {
        #[allow(unused)]
        impl<E: Entity, $($name: Component,)*> ComponentBundle<E> for ($($name,)*) {
            fn insert_entity(self, world: &mut SecondaryWorld<E>, id: EntityId<E>) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;

                $(
                    let column = world
                        .columns
                        .entry(TypeId::of::<$name>())
                        .or_insert(Box::new(SecondaryColumn::<E, $name>::new()));

                    let column: &mut SecondaryColumn<E, $name> =
                        column.downcast_mut().expect("Bug in SecondaryWorld");

                    column.insert(id, $name);
                )*
            }
        }
    };
}

smaller_tuples_too!(
    tuple_impl, F0, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15
);
