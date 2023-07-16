use std::{cell::RefCell, marker::PhantomData};

use crate::{Column, ColumnKey, Component};

// TODO: Debug, PartialEq, Eq, Hash, PartialOrd, Ord.
// https://github.com/rust-lang/rust/issues/26925
pub struct EntityKey<A>(ColumnKey, PhantomData<A>);

impl<A> Clone for EntityKey<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<A> Copy for EntityKey<A> {}

pub trait ArchetypeStorage {
    type Archetype: Archetype;

    fn column<C: Component>(&self) -> Option<&RefCell<Column<C>>>;

    fn spawn(&mut self, entity: Self::Archetype) -> EntityKey<Self::Archetype>;

    fn despawn(&mut self, key: EntityKey<Self::Archetype>) -> Option<Self::Archetype>;
}

pub trait Archetype: Sized {
    type Storage: ArchetypeStorage;
}

pub type Storage<A> = <A as Archetype>::Storage;
