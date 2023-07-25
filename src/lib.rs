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

mod archetype;
mod archetype_set;
mod borrow_checker;
mod column;

pub mod query;

pub use thunderdome;

pub use archetype::{Archetype, Entity, EntityColumns, EntityKey};
pub use archetype_set::{ArchetypeSet, ArchetypeSetFetch, EntityId, InArchetypeSet};
pub use column::Column;

#[doc(inline)]
pub use query::Query;

pub trait Component: 'static {}

impl<T> Component for T where T: 'static {}

// Hidden unstable symbols, needed for `stecs-derive`.
pub mod internal {
    pub use super::borrow_checker::BorrowChecker;
}
