use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DataEnum, DeriveInput, Error, Result};

use crate::utils::associated_ident;

// FIXME: Use `__stecs__` prefix for generic parameters consistently.

pub fn derive(input: &DeriveInput, data: &DataEnum, attrs: Vec<String>) -> Result<TokenStream2> {
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

    let derive_serde = if attrs.iter().any(|a| a == "serde") {
        quote! {
            #[derive(::stecs::serde::Serialize, ::stecs::serde::Deserialize)]
            #[serde(crate = "::stecs::serde")]
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        // Entity

        impl ::stecs::Entity for #ident {
            type Id = #ident_id;
            type Ref<#lifetime> = #ident_ref<#lifetime>;
            type RefMut<#lifetime> = #ident_ref_mut<#lifetime>;
            type WorldData = #ident_world_data;
            type Fetch<#lifetime> = #ident_ref_fetch<#lifetime>;
            type FetchMut<#lifetime> = #ident_ref_mut_fetch<#lifetime>;
            type FetchId<#lifetime> = #ident_id_fetch<#lifetime>;

            fn from_ref<'a>(entity: Self::Ref<'a>) -> Self {
                match entity {
                    #(
                        #ident_ref::#variant_idents(entity) => {
                            #ident::#variant_idents(
                                <#variant_tys as ::stecs::Entity>::from_ref(entity),
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

                fn id_to_outer(id: Self::Id) -> #ident_id {
                    #ident_id::#variant_idents(id)
                }
            }
        )*

        impl ::stecs::entity::EntityVariant<#ident> for #ident {
            fn into_outer(self) -> Self {
                self
            }

            fn id_to_outer(id: Self::Id) -> Self::Id {
                id
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
        #derive_serde
        #vis enum #ident_id {
            #(
                #variant_idents(<#variant_tys as ::stecs::Entity>::Id),
            )*
        }

        // Ref

        #[allow(non_camel_case_types)]
        #[derive(::std::clone::Clone)]
        #vis enum #ident_ref<#lifetime> {
            #(
                #variant_idents(<#variant_tys as ::stecs::Entity>::Ref<#lifetime>),
            )*
        }

        // RefMut

        #[allow(non_camel_case_types)]
        #vis enum #ident_ref_mut<#lifetime> {
            #(
                #variant_idents(<#variant_tys as ::stecs::Entity>::RefMut<#lifetime>),
            )*
        }

        // WorldData

        // TODO: Consider exposing the `WorldData` struct. In this case, convert
        // field names to snake case first.
        #[allow(non_snake_case, non_camel_case_types)]
        #[derive(::std::default::Default, ::std::clone::Clone)]
        #vis struct #ident_world_data {
            #(
                #variant_idents: <#variant_tys as ::stecs::Entity>::WorldData,
            )*
        }

        impl ::stecs::WorldData for #ident_world_data {
            type Entity = #ident;

            type Fetch<'w, F: ::stecs::query::fetch::Fetch + 'w> = #ident_world_fetch<'w, F>;

            fn spawn<E>(&mut self, entity: E) -> ::stecs::EntityId<E>
            where
                E: ::stecs::entity::EntityVariant<#ident>,
            {
                // FIXME: Ok, this is too crazy. All of this "just" so we can
                // return `EntityId<E>` rather than the outer `Id`.
                // TODO: Use a trait method for the id downcast.

                #(
                    if ::std::any::TypeId::of::<E>() == ::std::any::TypeId::of::<#variant_tys>() {
                        let #ident::#variant_idents(entity) =
                            <E as ::stecs::entity::EntityVariant<#ident>>::into_outer(entity)
                            else { panic!("bug in stecs") };

                        let id = ::stecs::WorldData::spawn(&mut self.#variant_idents, entity);
                        return ::stecs::archetype::adopt_entity_id_unchecked(id);
                    }
                )*

                assert_eq!(::std::any::TypeId::of::<E>(), ::std::any::TypeId::of::<#ident>());

                let id: #ident_id =
                    match <E as ::stecs::entity::EntityVariant<#ident>>::into_outer(entity) {
                        #(
                            #ident::#variant_idents(entity) => {
                                #ident_id::#variant_idents(self.#variant_idents.spawn(entity).get())
                            }
                        )*
                    };

                let id = ::stecs::EntityId::<#ident>::new(id);

                ::stecs::archetype::adopt_entity_id_unchecked(id)
            }

            fn despawn<E>(
                &mut self,
                id: ::stecs::EntityId<E>,
            ) -> ::std::option::Option<Self::Entity>
            where
                E: ::stecs::entity::EntityVariant<Self::Entity>,
            {
                match id.to_outer().get() {
                    #(
                        #ident_id::#variant_idents(id) => {
                            let id = ::stecs::EntityId::<#variant_tys>::new(id);
                            self.#variant_idents
                                .despawn(id)
                                .map(|entity| #ident::#variant_idents(entity))
                        }
                    )*
                }
            }

            fn spawn_at(
                &mut self,
                id: ::stecs::EntityId<Self::Entity>,
                entity: Self::Entity,
            ) -> ::std::option::Option<Self::Entity> {
                match (id.get(), entity) {
                    #(
                        (#ident_id::#variant_idents(id), #ident::#variant_idents(entity)) => {
                            self.#variant_idents.spawn_at(
                                ::stecs::EntityId::new(id),
                                entity,
                            )
                            .map(#ident::#variant_idents)
                        }
                    )*
                    _ => panic!("Incompatible EntityId and Entity variants in `spawn_at`"),
                }
            }

            fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
            where
                F: ::stecs::query::fetch::Fetch + 'w,
            {
                let mut fetch = #ident_world_fetch {
                    #(
                        #variant_idents:
                            <
                                ::stecs::world::EntityWorldData<#variant_tys>
                                as ::stecs::world::WorldData
                            >
                            ::fetch::<F>(&self.#variant_idents),
                    )*
                };

                fetch
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
        {
        }

        impl<'w, F> ::stecs::world::WorldFetch<'w, F> for #ident_world_fetch<'w, F>
        where
            F: ::stecs::query::fetch::Fetch,
        {
            type Data = #ident_world_data;
            type Iter = #world_fetch_iter;

            unsafe fn get<'a>(&self, id: #ident_id) -> ::std::option::Option<F::Item<'a>> {
                match id {
                    #(
                        #ident_id::#variant_idents(id) => {
                            // Safety: TODO
                            unsafe {
                                <
                                    ::stecs::world::EntityWorldFetch<'w, #variant_tys, F>
                                    as ::stecs::world::WorldFetch<F>
                                >
                                ::get(&self.#variant_idents, id)
                            }
                        }
                    )*
                }
            }

            fn iter(&mut self) -> Self::Iter {
                let iter = ::std::iter::empty();
                #(
                    let iter = ::std::iter::Iterator::chain(
                        iter,
                        <
                            ::stecs::world::EntityWorldFetch<'w, #variant_tys, F>
                            as ::stecs::world::WorldFetch<F>
                        >
                        ::iter(&mut self.#variant_idents),
                    );
                )*

                iter
            }

            fn len(&self) -> usize {
                let len = 0;
                #(
                    let len = len +
                        <
                            ::stecs::world::EntityWorldFetch<'w, #variant_tys, F>
                            as ::stecs::world::WorldFetch<F>
                        >
                        ::len(&self.#variant_idents);
                )*

                len
            }
        }


        // IdFetch

        #[allow(non_camel_case_types)]
        #[derive(
            ::std::clone::Clone,
            ::std::marker::Copy,
        )]
        #vis enum #ident_id_fetch<'w> {
            #(
                #variant_idents(<#variant_tys as ::stecs::entity::Entity>::FetchId<'w>),
            )*
        }

        unsafe impl<'w> ::stecs::query::fetch::Fetch for #ident_id_fetch<'w> {
            type Item<'a> = ::stecs::EntityId<#ident> where Self: 'a;

            fn new<A: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &A,
            ) -> ::std::option::Option<Self> {
                #(
                    if let ::std::option::Option::Some(fetch) =
                        ::stecs::query::fetch::Fetch::new(ids, columns) {
                        return ::std::option::Option::Some(#ident_id_fetch::#variant_idents(fetch))
                    }
                )*

                ::std::option::Option::None
            }

            fn len(&self) -> usize {
                match self {
                    #(
                        #ident_id_fetch::#variant_idents(fetch) => fetch.len(),
                    )*
                }
            }

            unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
            where
                Self: 'a,
            {
                ::stecs::EntityId::new(match self {
                    #(
                        #ident_id_fetch::#variant_idents(fetch) => {
                            #ident_id::#variant_idents(fetch.get(index).get())
                        }
                    )*
                })
            }

            fn check_borrows(checker: &mut ::stecs::query::borrow_checker::BorrowChecker) {
                #(
                    <#variant_tys as ::stecs::entity::Entity>::FetchId::<'w>::check_borrows(checker);
                )*
            }
        }

        // RefFetch

        #[allow(non_camel_case_types)]
        #[derive(
            ::std::clone::Clone,
            ::std::marker::Copy,
        )]
        #vis enum #ident_ref_fetch<'w> {
            #(
                #variant_idents(<#variant_tys as ::stecs::entity::Entity>::Fetch<'w>),
            )*
        }

        unsafe impl<'w> ::stecs::query::fetch::Fetch for #ident_ref_fetch<'w> {
            type Item<'a> = #ident_ref<'a> where Self: 'a;

            fn new<A: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &A,
            ) -> ::std::option::Option<Self> {
                #(
                    if let ::std::option::Option::Some(fetch) =
                        ::stecs::query::fetch::Fetch::new(ids, columns) {
                        return ::std::option::Option::Some(#ident_ref_fetch::#variant_idents(fetch));
                    }
                )*

                ::std::option::Option::None
            }

            fn len(&self) -> usize {
                match self {
                    #(
                        #ident_ref_fetch::#variant_idents(fetch) => fetch.len(),
                    )*
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

            fn check_borrows(checker: &mut ::stecs::query::borrow_checker::BorrowChecker) {
                #(
                    <#variant_tys as ::stecs::entity::Entity>::Fetch::<'w>::check_borrows(checker);
                )*
            }
        }

        impl<'q> ::stecs::Query for #ident_ref<'q> {
            type Fetch<'w> = #ident_ref_fetch<'w>;
        }

        impl<'q> ::stecs::QueryShared for #ident_ref<'q> {
        }

        // RefMutFetch

        #[allow(non_camel_case_types)]
        #[derive(
            ::std::clone::Clone,
            ::std::marker::Copy,
        )]
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
                #(
                    if let ::std::option::Option::Some(fetch) =
                        ::stecs::query::fetch::Fetch::new(ids, columns) {
                        return ::std::option::Option::Some(#ident_ref_mut_fetch::#variant_idents(fetch));
                    }
                )*

                ::std::option::Option::None
            }

            fn len(&self) -> usize {
                match self {
                    #(
                        #ident_ref_mut_fetch::#variant_idents(fetch) => fetch.len(),
                    )*
                }
            }

            unsafe fn get<'a>(&self, index: usize) -> Self::Item<'a>
            where
                Self: 'a,
            {
                match self {
                    #(
                        #ident_ref_mut_fetch::#variant_idents(fetch) => {
                            #ident_ref_mut::#variant_idents(fetch.get(index))
                        }
                    )*
                }
            }

            fn check_borrows(checker: &mut ::stecs::query::borrow_checker::BorrowChecker) {
                #(
                    <#variant_tys as ::stecs::entity::Entity>::FetchMut::<'w>::check_borrows(checker);
                )*
            }
        }

        impl<'q> ::stecs::Query for #ident_ref_mut<'q> {
            type Fetch<'w> = #ident_ref_mut_fetch<'w>;
        }
    })
}
