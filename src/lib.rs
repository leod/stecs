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
mod column;
mod query;

pub use archetype::{Archetype, Entity, EntityColumns, EntityKey};
pub use archetype_set::{ArchetypeSet, ArchetypeSetFetch, EntityId, InArchetypeSet};
pub use column::Column;
pub use query::{Fetch, Query};

pub trait Component: 'static {}

impl<T> Component for T where T: 'static {}
