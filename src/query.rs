use std::iter::Zip;

use crate::{
    archetype::{Column, ColumnIter},
    Archetype,
};

pub trait Query {
    type Iterator<'a>: Iterator<Item = Self>
    where
        Self: 'a;

    fn query<'a, A: Archetype>(storage: &'a A::Storage) -> Option<Self::Iterator<'a>>;
}

pub trait QueryMut {
    type Iterator<'a>: Iterator<Item = Self>
    where
        Self: 'a;

    fn query_mut<'a, A: Archetype>(storage: &'a mut A::Storage) -> Option<Self::Iterator<'a>>;
}

macro_rules! zip_type {
    () => { () };

    ($name:ident) => { ColumnIter<'a, $name> };

    ($name1:ident, $name2:ident) => {
        Zip<zip_type!($name1), zip_type!($name2)>
    };

    ($name:ident, $($rest:ident),*) => {
        Zip<ColumnIter<'a, $name>, zip_type!($($rest),*)>
    };
}

macro_rules! tuple_impl {
    ($($name:ident),*) => {

        #[allow(unused_parens)]
        impl<$($name: Query),*> Query for ($($name,)*) {
            type Iterator<'a> = zip_type!($($name),*) where Self: 'a;

            fn query<'a, A: Archetype>(storage: &'a A::Storage) -> Option<Self::Iterator<'a>> {
                $(
                    let Some($name) = A::column(storage) else { return None; };
                )*

                None
            }
        }
    };
}

smaller_tuples_too!(tuple_impl, F, E, D, C, B);
