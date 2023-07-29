use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use hashbrown::HashMap;

use crate::{Component, Entity};

use super::column::SecondaryColumn;

// TODO: Joining with the secondary world is currently pretty expensive, since
// it is one hash map lookup for each entity yielded from the primary query. We
// could try to improve this situation by having one hibitset per primary
// archetype in the secondary world.

pub struct SecondaryWorld<E: Entity>(HashMap<TypeId, Box<dyn Any>>, PhantomData<E>);

impl<E: Entity> Default for SecondaryWorld<E> {
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}

impl<E: Entity> SecondaryWorld<E> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn column<C: Component>(&self) -> Option<&SecondaryColumn<E, C>> {
        self.0
            .get(&TypeId::of::<C>())
            .and_then(|column| column.downcast_ref())
    }
}
