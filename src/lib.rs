/// Imagine macro parameters, but more like those Russian dolls.
///
/// Calls m!(A, B, C) and m!(A, B) for i.e. (m, A, B, C) where m is any macro,
/// for any number of parameters.
///
/// Copied from `hecs`.
macro_rules! smaller_tuples_too {
    ($m: ident, $ty: ident) => {};
    ($m: ident, $ty: ident, $($tt: ident),*) => {
        smaller_tuples_too!{$m, $($tt),*}
        $m!{$ty, $($tt),*}
    };
}

pub mod archetype;
pub mod column;
pub mod entity;
pub mod query;
pub mod world;

pub use thunderdome;

pub use stecs_derive::{ArchetypeSet, Entity};

#[doc(inline)]
pub use self::{
    entity::{Entity, EntityId, EntityRef, EntityRefMut},
    query::Query,
    world::{World, WorldData},
};
pub trait Component: 'static {}

impl<T> Component for T where T: 'static {}
