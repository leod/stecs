use std::cell::UnsafeCell;

use downcast_rs::Downcast;
use fxhash::FxHashMap;

use crate::{Component, Entity, Id};

pub struct SecondaryColumn<E: Entity, C>(FxHashMap<Id<E>, UnsafeCell<C>>);

impl<E: Entity, C> Default for SecondaryColumn<E, C> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<E: Entity, C> SecondaryColumn<E, C> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, id: Id<E>) -> Option<&UnsafeCell<C>> {
        self.0.get(&id)
    }

    pub fn insert(&mut self, id: Id<E>, component: C) {
        self.0.insert(id, component.into());
    }

    pub fn remove(&mut self, id: Id<E>) {
        self.0.remove(&id);
    }
}

pub trait AnySecondaryColumn<E: Entity>: Downcast + 'static {
    fn remove(&mut self, id: Id<E>);
}

downcast_rs::impl_downcast!(AnySecondaryColumn<E> where E: Entity);

impl<E: Entity, C: Component> AnySecondaryColumn<E> for SecondaryColumn<E, C> {
    fn remove(&mut self, id: Id<E>) {
        self.0.remove(&id);
    }
}
