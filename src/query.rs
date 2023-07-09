use std::iter::{Empty, Flatten, Map, Zip};

use crate::{Archetype, Column, ColumnIter, ColumnValues, Component, Storage};

pub trait Query<'a> {
    type Iter: Iterator<Item = Self> + 'a;

    fn query<A: Archetype>(storage: &'a Storage<A>) -> Option<Self::Iter>;
}

macro_rules! zip_type {
    () => { Empty<()> };

    ($lt:lifetime, $name:ty) => { <$name as Query>::Iter<$lt> };

    ($lt:lifetime, $name1:ty, $name2:ty) => {
        Zip<zip_type!($lt, $name1), zip_type!($lt, $name2)>
    };

    ($lt:lifetime, $name:ty, $($rest:ty),*) => {
        Zip<zip_type!($lt, $name), zip_type!($lt, $($rest),*)>
    };
}

impl<'a, C: Component> Query<'a> for &'a C {
    type Iter = ColumnValues<'a, C>;

    fn query<A: Archetype>(storage: &'a Storage<A>) -> Option<Self::Iter> {
        storage
            .column()
            .map(move |column| ColumnValues(column.iter()))
    }
}

/*
impl<'a, D: Query, E: Query> Query for (&'a D, &'a E) {
    type Iter<'b> = zip_type!('b, &'a D, &'a E)
    where
        Self: 'b;

    fn query<'b, A: Archetype>(storage: &'b Storage<A>) -> Option<Self::Iter<'b>>
    where
        Self: 'b,
    {
        todo!()
    }

*/

/*
macro_rules! tuple_impl {
    ($($name:ident),*) => {
        #[allow(unused_parens)]
        impl<$($name: Query),*> Query for ($($name,)*) {
            type Iter<'a> = zip_type!($($name),*) where Self: 'a;

            fn query<'a, A: Archetype>(storage: &'a Storage<A>) -> Option<Self::Iter<'a>> {
                $(
                    let Some($name) = storage.column::<A>() else { return None; };
                )*

                None
            }
        }
    };
}

smaller_tuples_too!(tuple_impl, F);
*/
