/// Imagine macro parameters, but more like those Russian dolls.
///
/// Calls m!(A, B, C), m!(A, B), m!(B), and m!() for i.e. (m, A, B, C)
/// where m is any macro, for any number of parameters.
///
/// Copied from `hecs`.
macro_rules! smaller_tuples_too {
    ($m: ident, $ty: ident) => {
        $m!{}
        $m!{$ty}
    };
    ($m: ident, $ty: ident, $($tt: ident),*) => {
        smaller_tuples_too!{$m, $($tt),*}
        $m!{$ty, $($tt),*}
    };
}

mod archetype;
mod query;
mod world;

pub mod arena;

pub use archetype::{Archetype, EntityIndex};
pub use arena::Arena;
pub use query::{Getter, GetterIter, Query};
pub use world::{Entity, EntityId, World};

pub trait Component: 'static {}

impl<T> Component for T where T: 'static {}
