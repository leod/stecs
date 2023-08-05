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

pub mod archetype;
pub mod column;
pub mod entity;
pub mod query;
pub mod secondary;
pub mod world;

pub use thunderdome;

pub use stecs_derive::{Entity, Query};

#[doc(inline)]
pub use self::{
    entity::{Entity, EntityId, EntityRef, EntityRefMut},
    query::{Or, Query, QueryShared, With, Without},
    secondary::{query::SecondaryQuery, query::SecondaryQueryShared, world::SecondaryWorld},
    world::{World, WorldData},
};

pub trait Component: 'static {}

impl<T> Component for T where T: 'static {}
