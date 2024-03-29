use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DataEnum, DeriveInput, Error, Result};

use crate::utils::{associated_ident, get_attr_derives, Derives};

// FIXME: Use `__stecs__` prefix for generic parameters consistently.

pub fn derive(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream2> {
    let ident = &input.ident;
    let vis = &input.vis;

    // We need to generate an army of eight associated types. These are their names.
    let ident_id = associated_ident(ident, "Id");
    let ident_ref = associated_ident(ident, "Ref");
    let ident_ref_mut = associated_ident(ident, "RefMut");
    let ident_world_data = associated_ident(ident, "WorldData");
    let ident_id_fetch = associated_ident(ident, "IdFetch");
    let ident_ref_fetch = associated_ident(ident, "RefFetch");
    let ident_ref_mut_fetch = associated_ident(ident, "RefMutFetch");
    let ident_world_fetch = associated_ident(ident, "WorldFetch");

    let Derives {
        id_derives,
        world_data_derives,
        ref_derives,
        ..
    } = get_attr_derives(&input.attrs)?;

    // As an example, our input looks like this:
    // ```
    // enum Entity {
    //     Player(Foo),
    //     Enemy(Enemy),
    // }
    // ```
    //
    // The following code extracts the identifiers (e.g. `Player`) and their
    // types (e.g. `Foo`).
    let variant_idents: Vec<_> = data
        .variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();

    let variant_tys: Vec<_> = data
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                Ok(fields.unnamed.iter().collect::<Vec<_>>()[0].ty.clone())
            }
            _ => Err(Error::new_spanned(
                ident,
                "For derive(Entity) on enums, each variant must have exactly one unnamed field",
            )),
        })
        .collect::<Result<_>>()?;

    // This is the iterator type that is used to execute queries over the
    // `WorldData` associated withour `Entity` enum. It is obtained by chaining
    // the iterators of each of our variant types. The variant types in turn can
    // be either structs (leafs) or enums (nodes).
    let world_fetch_iter = variant_tys
        .iter()
        .map(|ty| quote! { ::stecs::world::EntityWorldFetchIter<'w, #ty, F> })
        .fold(
            quote! { ::std::iter::Empty<F> },
            |chain, ty| quote! { ::std::iter::Chain<#chain, #ty> },
        );

    let lifetime: syn::Lifetime = syn::parse_str("'__stecs__f").unwrap();
    let type_param: syn::TypeParam = syn::parse_str("__stecs__F").unwrap();

    // TODO: We could probably remove this requirement.
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            ident,
            "derive(Entity) for enums must not have any generics",
        ));
    }

    Ok(quote! {
        // Entity

        impl ::stecs::Entity for #ident {
            type Id = #ident_id;
            type Borrow<#lifetime> = #ident_ref<#lifetime>;
            type BorrowMut<#lifetime> = #ident_ref_mut<#lifetime>;
            type WorldData = #ident_world_data;
            type Fetch<#lifetime> = #ident_ref_fetch<#lifetime>;
            type FetchMut<#lifetime> = #ident_ref_mut_fetch<#lifetime>;
            type FetchId<#lifetime> = #ident_id_fetch<#lifetime>;
        }

        // CloneEntityFromRef

        impl ::stecs::CloneEntityFromRef for #ident
        where
            // https://github.com/rust-lang/rust/issues/48214#issuecomment-1150463333
            #(for<'__stecs__a> #variant_tys: ::stecs::CloneEntityFromRef,)*
        {
            fn clone_entity_from_ref(entity: Self::Borrow<'_>) -> Self
            {
                match entity {
                    #(
                        #ident_ref::#variant_idents(entity) => {
                            #ident::#variant_idents(
                                <#variant_tys as ::stecs::CloneEntityFromRef>::clone_entity_from_ref(
                                    entity,
                                ),
                            )
                        }
                    )*
                }
            }
        }

        // EntityVariant

        #(
            impl ::stecs::entity::EntityVariant<#ident> for #variant_tys {
                fn into_outer(self) -> #ident {
                    #ident::#variant_idents(self)
                }

                fn spawn(self, data: &mut #ident_world_data) -> ::stecs::Id<Self> {
                    use ::stecs::WorldData;

                    data.#variant_idents.spawn(self)
                }

                fn id_to_outer(id: Self::Id) -> #ident_id {
                    #ident_id::#variant_idents(id)
                }

                fn try_id_from_outer(id: #ident_id) -> ::std::option::Option<Self::Id> {
                    if let #ident_id::#variant_idents(id) = id {
                        Some(id)
                    } else {
                        None
                    }
                }
            }
        )*

        impl ::stecs::entity::EntityVariant<#ident> for #ident {
            fn into_outer(self) -> Self {
                self
            }

            fn spawn(self, data: &mut #ident_world_data) -> ::stecs::Id<Self> {
                use ::stecs::WorldData;

                match self {
                    #(
                        #ident::#variant_idents(entity) =>
                            data.#variant_idents.spawn(entity).to_outer(),
                    )*
                }
            }

            fn id_to_outer(id: Self::Id) -> Self::Id {
                id
            }

            fn try_id_from_outer(id: Self::Id) -> ::std::option::Option<Self::Id> {
                Some(id)
            }
        }

        // Id

        #[allow(non_camel_case_types)]
        #[derive(
            ::std::clone::Clone,
            ::std::marker::Copy,
            ::std::fmt::Debug,
            ::std::cmp::PartialEq,
            ::std::cmp::Eq,
            ::std::cmp::PartialOrd,
            ::std::cmp::Ord,
            ::std::hash::Hash,
        )]
        #id_derives
        #vis enum #ident_id {
            #(#variant_idents(<#variant_tys as ::stecs::Entity>::Id),)*
        }

        // Ref

        #[allow(non_camel_case_types)]
        #[derive(::std::clone::Clone)]
        #ref_derives
        #vis enum #ident_ref<#lifetime> {
            #(#variant_idents(<#variant_tys as ::stecs::Entity>::Borrow<#lifetime>),)*
        }

        // RefMut

        #[allow(non_camel_case_types)]
        #vis enum #ident_ref_mut<#lifetime> {
            #(#variant_idents(<#variant_tys as ::stecs::Entity>::BorrowMut<#lifetime>),)*
        }

        // WorldData

        // TODO: Consider exposing the `WorldData` struct. In this case, convert
        // field names to snake case first.
        #[allow(non_snake_case, non_camel_case_types)]
        #[derive(::std::default::Default)]
        #world_data_derives
        #vis struct #ident_world_data {
            #(#variant_idents: <#variant_tys as ::stecs::Entity>::WorldData,)*
        }

        impl ::stecs::WorldData for #ident_world_data {
            type Entity = #ident;
            type Fetch<'w, F: ::stecs::query::fetch::Fetch + 'w> = #ident_world_fetch<'w, F>;

            fn spawn<E>(&mut self, entity: E) -> ::stecs::Id<E>
            where
                E: ::stecs::entity::EntityVariant<#ident>,
            {
                E::spawn(entity, self)
            }

            fn despawn<E>(
                &mut self,
                id: ::stecs::Id<E>,
            ) -> ::std::option::Option<Self::Entity>
            where
                E: ::stecs::entity::EntityVariant<Self::Entity>,
            {
                match id.to_outer().get() {
                    #(
                        #ident_id::#variant_idents(id) => {
                            self.#variant_idents
                                .despawn(::stecs::Id::<#variant_tys>::new(id))
                                .map(|entity| #ident::#variant_idents(entity))
                        }
                    )*
                }
            }

            fn spawn_at(
                &mut self,
                id: ::stecs::Id<Self::Entity>,
                entity: Self::Entity,
            ) -> ::std::option::Option<Self::Entity> {
                match (id.get(), entity) {
                    #(
                        (#ident_id::#variant_idents(id), #ident::#variant_idents(entity)) => {
                            self.#variant_idents
                                .spawn_at(::stecs::Id::new(id), entity)
                                .map(#ident::#variant_idents)
                        }
                    )*
                    _ => panic!("Incompatible Id and Entity variants in `spawn_at`"),
                }
            }

            fn contains(&self, id: ::stecs::Id<Self::Entity>) -> bool {
                match id.get() {
                    #(
                        #ident_id::#variant_idents(id) => {
                            self.#variant_idents
                                .contains(::stecs::Id::<#variant_tys>::new(id))
                        }
                    )*
                }
            }

            fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
            where
                F: ::stecs::query::fetch::Fetch + 'w,
            {
                #ident_world_fetch {
                    #(#variant_idents: self.#variant_idents.fetch::<F>(),)*
                }
            }
        }

        // WorldFetch

        #[allow(non_snake_case, non_camel_case_types)]
        #vis struct #ident_world_fetch<#lifetime, #type_param>
        where
            #type_param: ::stecs::query::fetch::Fetch + #lifetime,
        {
            #(
                #variant_idents: ::stecs::world::EntityWorldFetch<
                    #lifetime,
                    #variant_tys,
                    #type_param,
                >,
            )*
        }

        impl<#lifetime, #type_param> ::std::clone::Clone
        for #ident_world_fetch<#lifetime, #type_param>
        where
            #type_param: ::stecs::query::fetch::Fetch + #lifetime,
        {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<#lifetime, #type_param> ::std::marker::Copy
        for #ident_world_fetch<#lifetime, #type_param>
        where
            #type_param: ::stecs::query::fetch::Fetch + #lifetime,
        {}

        impl<'w, F> ::stecs::world::WorldFetch<'w, F> for #ident_world_fetch<'w, F>
        where
            F: ::stecs::query::fetch::Fetch,
        {
            type Data = #ident_world_data;
            type Iter = #world_fetch_iter;

            #[inline]
            unsafe fn get<'a>(&self, id: #ident_id) -> ::std::option::Option<F::Item<'a>> {
                // Safety: TODO
                match id {
                    #(#ident_id::#variant_idents(id) => unsafe { self.#variant_idents.get(id) },)*
                }
            }

            fn iter(&mut self) -> Self::Iter {
                let iter = ::std::iter::empty();
                #(let iter = ::std::iter::Iterator::chain(iter, self.#variant_idents.iter());)*

                iter
            }

            #[inline]
            fn len(&self) -> usize {
                let mut len = 0;
                #(len += self.#variant_idents.len();)*

                len
            }
        }


        // IdFetch

        #[allow(non_camel_case_types)]
        #[derive(::std::clone::Clone, ::std::marker::Copy)]
        #vis enum #ident_id_fetch<'w> {
            #(#variant_idents(<#variant_tys as ::stecs::entity::Entity>::FetchId<'w>),)*
        }

        unsafe impl<'w> ::stecs::query::fetch::Fetch for #ident_id_fetch<'w> {
            type Item<'a> = ::stecs::Id<#ident> where Self: 'a;

            fn new<A: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &A,
            ) -> ::std::option::Option<Self> {
                let mut result = None;
                #(
                    result = result.or_else(|| ::stecs::query::fetch::Fetch::new(ids, columns).map(
                        #ident_id_fetch::#variant_idents,
                    ));
                )*

                result
            }

            fn len(&self) -> usize {
                match self {
                    #(#ident_id_fetch::#variant_idents(fetch) => fetch.len(),)*
                }
            }

            unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
            where
                Self: 'a,
            {
                ::stecs::Id::new(match self {
                    #(
                        #ident_id_fetch::#variant_idents(fetch) =>
                            #ident_id::#variant_idents(fetch.get(index).get()),
                    )*
                })
            }
        }

        // RefFetch

        #[allow(non_camel_case_types)]
        #[derive(::std::clone::Clone, ::std::marker::Copy)]
        #vis enum #ident_ref_fetch<'w> {
            #(#variant_idents(<#variant_tys as ::stecs::entity::Entity>::Fetch<'w>),)*
        }

        unsafe impl<'w> ::stecs::query::fetch::Fetch for #ident_ref_fetch<'w> {
            type Item<'a> = #ident_ref<'a> where Self: 'a;

            fn new<A: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &A,
            ) -> ::std::option::Option<Self> {
                let mut result = None;
                #(
                    result = result.or_else(|| ::stecs::query::fetch::Fetch::new(ids, columns).map(
                        #ident_ref_fetch::#variant_idents,
                    ));
                )*

                result
            }

            fn len(&self) -> usize {
                match self {
                    #(#ident_ref_fetch::#variant_idents(fetch) => fetch.len(),)*
                }
            }

            unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
            where
                Self: 'a,
            {
                match self {
                    #(
                        #ident_ref_fetch::#variant_idents(fetch) => {
                            #ident_ref::#variant_idents(fetch.get(index))
                        }
                    )*
                }
            }
        }

        unsafe impl<'q> ::stecs::Query for #ident_ref<'q> {
            type Fetch<'w> = #ident_ref_fetch<'w>;

            fn for_each_borrow(mut f: impl FnMut(::std::any::TypeId, bool)) {
                #(<#variant_tys as ::stecs::Entity>::Borrow::<'q>::for_each_borrow(&mut f);)*
            }
        }

        unsafe impl<'q> ::stecs::QueryShared for #ident_ref<'q> {}

        // RefMutFetch

        #[allow(non_camel_case_types)]
        #[derive(::std::clone::Clone, ::std::marker::Copy)]
        #vis enum #ident_ref_mut_fetch<'w> {
            #(
                #variant_idents(<#variant_tys as ::stecs::entity::Entity>::FetchMut<'w>),
            )*
        }

        unsafe impl<'w> ::stecs::query::fetch::Fetch for #ident_ref_mut_fetch<'w> {
            type Item<'a> = #ident_ref_mut<'a> where Self: 'a;

            fn new<A: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &A,
            ) -> ::std::option::Option<Self> {
                let mut result = None;
                #(
                    result = result.or_else(|| ::stecs::query::fetch::Fetch::new(ids, columns).map(
                        #ident_ref_mut_fetch::#variant_idents,
                    ));
                )*

                result
            }

            fn len(&self) -> usize {
                match self {
                    #(#ident_ref_mut_fetch::#variant_idents(fetch) => fetch.len(),)*
                }
            }

            unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
            where
                Self: 'a,
            {
                match self {
                    #(
                        #ident_ref_mut_fetch::#variant_idents(fetch) =>
                            #ident_ref_mut::#variant_idents(fetch.get(index)),
                    )*
                }
            }
        }

        unsafe impl<'q> ::stecs::Query for #ident_ref_mut<'q> {
            type Fetch<'w> = #ident_ref_mut_fetch<'w>;

            fn for_each_borrow(mut f: impl FnMut(::std::any::TypeId, bool)) {
                let mut borrows = ::stecs::fxhash::FxHashSet::default();

                // NOTE: We are borrowing components mutably here. However, the
                // entity enum variants might have overlapping component type
                // sets. We need to deduplicate the component types here, so
                // that the borrow check (`assert_borrow`) does not raise an
                // error.

                // TODO: Collecting into a hash map is likely to cause per-query
                // runtime overhead. Is there a way to write this that enables
                // the compiler to optimize it out when used in `assert_borrow`?
                #(
                    <#variant_tys as ::stecs::Entity>::BorrowMut::<'q>::for_each_borrow(
                        |borrow, _| {
                            borrows.insert(borrow);
                        }
                    );
                )*

                for borrow in borrows {
                    f(borrow, true);
                }
            }
        }
    })
}
