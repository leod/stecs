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

mod borrow_checker;
mod column;

pub mod archetype;
pub mod archetype_set;
pub mod entity;
pub mod query;

pub use thunderdome;

pub use stecs_derive::Entity;

#[doc(inline)]
pub use self::{
    archetype::Archetype,
    archetype_set::{ArchetypeSet, EntityId},
    entity::{Entity, EntityKey, EntityRef, EntityRefMut},
    query::Query,
};
pub trait Component: 'static {}

impl<T> Component for T where T: 'static {}

// Hidden unstable symbols, needed for `stecs-derive`.
#[doc(hidden)]
pub mod internal {
    pub use super::{
        borrow_checker::BorrowChecker,
        column::{Column, ColumnRawParts, ColumnRawPartsMut},
    };
}
