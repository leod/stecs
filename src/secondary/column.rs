use std::cell::UnsafeCell;

use downcast_rs::Downcast;
use hashbrown::HashMap;

use crate::{Component, Entity, EntityId};

pub struct SecondaryColumn<E: Entity, C>(HashMap<EntityId<E>, UnsafeCell<C>>);

impl<E: Entity, C> SecondaryColumn<E, C> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get(&self, id: EntityId<E>) -> Option<&UnsafeCell<C>> {
        self.0.get(&id)
    }

    pub fn insert(&mut self, id: EntityId<E>, component: C) {
        self.0.insert(id, component.into());
    }

    pub fn remove(&mut self, id: EntityId<E>) {
        self.0.remove(&id);
    }
}

pub trait AnySecondaryColumn<E: Entity>: Downcast + 'static {
    fn remove(&mut self, id: EntityId<E>);
}

downcast_rs::impl_downcast!(AnySecondaryColumn<E> where E: Entity);

impl<E: Entity, C: Component> AnySecondaryColumn<E> for SecondaryColumn<E, C> {
    fn remove(&mut self, id: EntityId<E>) {
        self.0.remove(&id);
    }
}
