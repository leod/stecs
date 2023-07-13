use frunk::prelude::HList;

use crate::Component;

pub unsafe trait Archetype: Sized {
    type Components: HList;

    fn offset_of<C: Component>() -> Option<usize>;
}
