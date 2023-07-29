use std::cell::UnsafeCell;

use hashbrown::HashMap;

use crate::EntityId;

pub type SecondaryColumn<E, C> = HashMap<EntityId<E>, UnsafeCell<C>>;
