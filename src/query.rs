use std::iter::{Empty, Map, Zip};

use crate::{
    archetype::{Column, ColumnIter},
    Archetype,
};

pub trait Query {
    type Iter<'a>: Iterator<Item = Self>
    where
        Self: 'a;

    fn query<'a, A: Archetype>(storage: &'a A::Storage) -> Option<Self::Iter<'a>>;
}

pub trait QueryMut {
    type Iter<'a>: Iterator<Item = Self>
    where
        Self: 'a;

    fn query_mut<'a, A: Archetype>(storage: &'a mut A::Storage) -> Option<Self::Iter<'a>>;
}

macro_rules! zip_type {
    () => { Empty<()> };

    ($name:ident) => { ColumnIter<'a, $name> };

    ($name1:ident, $name2:ident) => {
        Zip<zip_type!($name1), zip_type!($name2)>
    };

    ($name:ident, $($rest:ident),*) => {
        Zip<ColumnIter<'a, $name>, zip_type!($($rest),*)>
    };
}

pub struct Iter<T>(T);

macro_rules! tuple_impl {
    ($($name:ident),*) => {
        impl<$($name: Query,)*> Iterator for Iter<($(ColumnIter<$name>,)*)> {
            type Item = ($($name,)*);

            fn next(&mut self) -> Option<Self::Item> {
                let Iter(($($name,)*)) = self;

                $(
                    let Some((_, $name)) = $name.next() else { return None; };
                )*

                Some(($($name,)*))
            }
        }

        #[allow(unused_parens)]
        impl<$($name: Query),*> Query for ($($name,)*) {
            type Iter<'a> = zip_type!($($name),*) where Self: 'a;

            fn query<'a, A: Archetype>(storage: &'a A::Storage) -> Option<Self::Iterator<'a>> {
                $(
                    let Some($name) = A::column(storage) else { return None; };
                )*

                None
            }
        }
    };
}

smaller_tuples_too!(tuple_impl, F);
