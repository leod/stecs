use frunk::prelude::HList;

use crate::Component;

pub type EntityIndex = thunderdome::Index;

pub unsafe trait Archetype {
    type Components: HList;

    fn offset_of<C: Component>() -> Option<usize>;
}
