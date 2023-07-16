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
//mod query;
mod column;

pub use archetype::{Archetype, ArchetypeStorage, EntityKey, Storage};
pub use archetype_set::{ArchetypeInSet, ArchetypeSet, Entity, EntityId};
//pub use query::{BorrowChecker, EntityIdGetter, Getter, GetterIter, Query};
pub use column::{Column, ColumnKey};

pub trait Component: 'static {}

impl<T> Component for T where T: 'static {}
