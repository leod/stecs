use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DataEnum, DataStruct, DeriveInput, Error, Result};

use crate::utils::{generics_with_new_lifetime, members_as_idents, struct_fields};

pub fn derive(input: DeriveInput) -> Result<TokenStream2> {
    match input.data {
        syn::Data::Struct(ref data) => derive_struct(&input, data),
        syn::Data::Enum(ref data) => derive_enum(&input, data),
        _ => Err(Error::new_spanned(
            input.ident,
            "derive(Entity) only supports structs and enums",
        )),
    }
}

// FIXME: Use `__stecs__` prefix for generic parameters consistently.

fn derive_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream2> {
    let ident = &input.ident;
    let vis = &input.vis;

    let ident_columns = syn::Ident::new(&format!("{ident}StecsInternalColumns"), ident.span());
    let ident_ref = syn::Ident::new(&format!("{ident}StecsInternalRef"), ident.span());
    let ident_ref_fetch = syn::Ident::new(&format!("{ident}StecsInternalRefFetch"), ident.span());
    let ident_ref_mut = syn::Ident::new(&format!("{ident}StecsInternalRefMut"), ident.span());
    let ident_ref_mut_fetch =
        syn::Ident::new(&format!("{ident}StecsInternalRefMutFetch"), ident.span());

    let (field_tys, field_members) = struct_fields(&data.fields);
    let field_idents = members_as_idents(&field_members);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetime: syn::Lifetime = syn::parse_str("'__stecs__f").unwrap();
    let generics_with_lifetime = generics_with_new_lifetime(&input.generics, &lifetime);
    let (impl_generics_with_lifetime, ty_generics_with_lifetime, where_clause_with_lifetime) =
        generics_with_lifetime.split_for_impl();

    let from_ref = match &data.fields {
        syn::Fields::Named(_) => quote! {
            Self {
                #(
                    #field_members: ::std::clone::Clone::clone(entity.#field_idents),
                )*
            }
        },
        syn::Fields::Unnamed(_) => quote! {
            Self(
                #(
                    ::std::clone::Clone::clone(entity.#field_idents),
                )*
            )
        },
        syn::Fields::Unit => quote! {Self},
    };

    Ok(quote! {
        // Columns

        // TODO: Provide a way to derive traits for the column struct.
        // Otherwise, we lose the ability to derive things for our World.
        #[allow(unused)]
        #[derive(::std::clone::Clone)]
        #vis struct #ident_columns #impl_generics #where_clause {
            #(
                #field_idents: ::std::cell::RefCell<::stecs::column::Column<#field_tys>>
            ),*
        }

        impl #impl_generics ::std::default::Default
        for #ident_columns #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(
                        #field_idents: ::std::default::Default::default()
                    ),*
                }
            }
        }

        impl #impl_generics ::stecs::entity::Columns
        for #ident_columns #ty_generics #where_clause {
            type Entity = #ident #ty_generics;

            fn column<__stecs__C: ::stecs::Component>(
                &self,
            )
            -> ::std::option::Option<&::std::cell::RefCell<::stecs::column::Column<__stecs__C>>>
            {
                #(
                    if ::std::any::TypeId::of::<__stecs__C>() ==
                           ::std::any::TypeId::of::<#field_tys>() {
                        return (&self.#field_idents as &dyn ::std::any::Any).downcast_ref();
                    }
                )*

                ::std::option::Option::None
            }

            fn push(&mut self, entity: Self::Entity) {
                #(
                    self.#field_idents.borrow_mut().push(entity.#field_members)
                );*
            }

            fn remove(&mut self, index: usize) -> Self::Entity {
                #ident {
                    #(
                        #field_members: self.#field_idents.borrow_mut().remove(index)
                    ),*
                }
            }

            fn new_fetch<'__stecs__w, #lifetime>(
                &'__stecs__w self,
                len: usize,
            ) -> <Self::Entity as ::stecs::entity::Entity>::Fetch<'__stecs__w>
            where
                '__stecs__w: #lifetime,
            {
                #(
                    ::std::debug_assert_eq!(len, self.#field_idents.borrow().len());
                )*

                #ident_ref_fetch {
                    __stecs__len: len,
                    #(
                        #field_idents: self.#field_idents.borrow().as_raw_parts(),
                    )*
                }
            }

            fn new_fetch_mut<'__stecs__w, #lifetime>(
                &'__stecs__w self,
                len: usize,
            ) -> <Self::Entity as ::stecs::entity::Entity>::FetchMut<'__stecs__w>
            where
                '__stecs__w: #lifetime,
            {
                #(
                    ::std::debug_assert_eq!(len, self.#field_idents.borrow().len());
                )*

                #ident_ref_mut_fetch {
                    __stecs__len: len,
                    #(
                        #field_idents: self.#field_idents.borrow_mut().as_raw_parts_mut(),
                    )*
                }
            }
        }

        // RefFetch

        #[allow(unused, non_snake_case)]
        #vis struct #ident_ref_fetch #impl_generics #where_clause {
            __stecs__len: usize,
            #(
                #field_idents: ::stecs::column::ColumnRawParts<#field_tys>
            ),*
        }

        impl #impl_generics ::std::clone::Clone
        for #ident_ref_fetch #ty_generics #where_clause {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl #impl_generics ::std::marker::Copy
        for #ident_ref_fetch #ty_generics #where_clause {
        }

        unsafe impl #impl_generics ::stecs::query::fetch::Fetch
        for #ident_ref_fetch #ty_generics #where_clause {
            type Item<#lifetime> = #ident_ref #ty_generics_with_lifetime;

            fn new<__stecs__A: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &__stecs__A,
            ) -> ::std::option::Option<Self>
            {
                if ::std::any::TypeId::of::<__stecs__A>() ==
                       ::std::any::TypeId::of::<#ident_columns #ty_generics>() {
                    let columns: &#ident_columns #ty_generics =
                        (columns as &dyn ::std::any::Any).downcast_ref().unwrap();

                    ::std::option::Option::Some(
                        <#ident_columns #ty_generics as ::stecs::entity::Columns>
                            ::new_fetch(columns, ids.len()),
                    )
                } else {
                    ::std::option::Option::None
                }
            }

            fn len(&self) -> usize {
                self.__stecs__len
            }

            unsafe fn get<#lifetime>(&self, index: usize) -> Self::Item<#lifetime> {
                ::std::assert!(index < self.len());

                #ident_ref {
                    #(
                        #field_idents: &*self.#field_idents.ptr.add(index),
                    )*
                    __stecs__phantom: ::std::marker::PhantomData,
                }
            }

            fn check_borrows(checker: &mut ::stecs::query::borrow_checker::BorrowChecker) {
                #(
                    checker.borrow::<#field_tys>();
                )*
            }
        }

        impl #impl_generics_with_lifetime ::stecs::Query
        for #ident_ref #ty_generics_with_lifetime #where_clause {
            type Fetch<'__stecs__w> = #ident_ref_fetch #ty_generics;
        }

        impl #impl_generics_with_lifetime ::stecs::QueryShared
        for #ident_ref #ty_generics_with_lifetime #where_clause {
        }

        // Ref

        // FIXME: This should be a tuple struct for tuple structs.
        #[allow(unused, non_snake_case)]
        #[derive(::std::clone::Clone)]
        #vis struct #ident_ref #impl_generics_with_lifetime #where_clause_with_lifetime {
            #(
                #field_idents: &#lifetime #field_tys,
            )*
            __stecs__phantom: ::std::marker::PhantomData<&#lifetime ()>,
        }

        // RefMutFetch

        #[allow(unused, non_snake_case)]
        #vis struct #ident_ref_mut_fetch #impl_generics #where_clause {
            __stecs__len: usize,
            #(
                #field_idents: ::stecs::column::ColumnRawPartsMut<#field_tys>
            ),*
        }

        impl #impl_generics ::std::clone::Clone
        for #ident_ref_mut_fetch #ty_generics #where_clause {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl #impl_generics ::std::marker::Copy
        for #ident_ref_mut_fetch #ty_generics #where_clause {
        }

        unsafe impl #impl_generics ::stecs::query::fetch::Fetch
        for #ident_ref_mut_fetch #ty_generics #where_clause {
            type Item<#lifetime> = #ident_ref_mut #ty_generics_with_lifetime;

            fn new<__stecs__A: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &__stecs__A,
            ) -> ::std::option::Option<Self>
            {
                if ::std::any::TypeId::of::<__stecs__A>() ==
                       ::std::any::TypeId::of::<#ident_columns #ty_generics>() {
                    let columns: &#ident_columns #ty_generics =
                        (columns as &dyn ::std::any::Any).downcast_ref().unwrap();

                    ::std::option::Option::Some(
                        <#ident_columns #ty_generics as ::stecs::entity::Columns>
                            ::new_fetch_mut(columns, ids.len()),
                    )
                } else {
                    ::std::option::Option::None
                }
            }

            fn len(&self) -> usize {
                self.__stecs__len
            }

            unsafe fn get<#lifetime>(&self, index: usize) -> Self::Item<#lifetime> {
                ::std::assert!(index < self.len());

                #ident_ref_mut {
                    #(
                        #field_idents: &mut *self.#field_idents.ptr.add(index),
                    )*
                    __stecs__phantom: ::std::marker::PhantomData,
                }
            }

            fn check_borrows(checker: &mut ::stecs::query::borrow_checker::BorrowChecker) {
                #(
                    checker.borrow_mut::<#field_tys>();
                )*
            }
        }

        impl #impl_generics_with_lifetime ::stecs::Query
        for #ident_ref_mut #ty_generics_with_lifetime #where_clause {
            type Fetch<'__stecs__w> = #ident_ref_mut_fetch #ty_generics;
        }

        // RefMut

        // FIXME: This should be a tuple struct for tuple structs.
        #[allow(unused, non_snake_case)]
        #vis struct #ident_ref_mut #impl_generics_with_lifetime #where_clause_with_lifetime {
            #(
                #field_idents: &#lifetime mut #field_tys,
            )*
            __stecs__phantom: ::std::marker::PhantomData<&#lifetime mut ()>,
        }

        // EntityVariant

        impl #impl_generics ::stecs::entity::EntityVariant<#ident #ty_generics>
        for #ident #ty_generics #where_clause {
            fn into_outer(self) -> Self {
                self
            }

            fn id_to_outer(id: Self::Id) -> Self::Id {
                id
            }
        }

        // EntityStruct

        impl #impl_generics ::stecs::entity::EntityStruct for #ident #ty_generics #where_clause {
            type Columns = #ident_columns #ty_generics;
        }

        // Entity

        impl #impl_generics ::stecs::Entity for #ident #ty_generics #where_clause {
            type Id = ::stecs::archetype::EntityKey<Self>;
            type Ref<#lifetime> = #ident_ref #ty_generics_with_lifetime;
            type RefMut<#lifetime> = #ident_ref_mut #ty_generics_with_lifetime;
            type Fetch<'__stecs__w> = #ident_ref_fetch #ty_generics;
            type FetchMut<'__stecs__w> = #ident_ref_mut_fetch #ty_generics;
            type FetchId<'__stecs__w> = ::stecs::query::fetch::EntityKeyFetch<#ident #ty_generics>;
            type WorldData = ::stecs::archetype::Archetype<#ident_columns #ty_generics>;

            fn from_ref<'f>(entity: Self::Ref<'f>) -> Self {
                #from_ref
            }
        }
    })
}

fn derive_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream2> {
    let ident = &input.ident;
    let vis = &input.vis;

    let ident_id = syn::Ident::new(&format!("{ident}StecsInternalId"), ident.span());
    let ident_id_fetch = syn::Ident::new(&format!("{ident}StecsInternalIdFetch"), ident.span());
    let ident_ref = syn::Ident::new(&format!("{ident}StecsInternalRef"), ident.span());
    let ident_ref_fetch = syn::Ident::new(&format!("{ident}StecsInternalRefFetch"), ident.span());
    let ident_ref_mut = syn::Ident::new(&format!("{ident}StecsInternalRefMut"), ident.span());
    let ident_ref_mut_fetch =
        syn::Ident::new(&format!("{ident}StecsInternalRefMutFetch"), ident.span());
    let ident_world_fetch =
        syn::Ident::new(&format!("{ident}StecsInternalWorldFetch"), ident.span());
    let ident_world_data = syn::Ident::new(&format!("{ident}StecsInternalWorldData"), ident.span());

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

    let world_fetch_iter = variant_tys
        .iter()
        .map(|ty| {
            quote! {
                <
                    <
                        <
                            #ty
                            as ::stecs::Entity
                        >::WorldData
                        as ::stecs::world::WorldData
                    >::Fetch<'w, F>
                    as ::stecs::world::WorldFetch<
                        'w,
                        <
                            #ty
                            as ::stecs::Entity
                        >::WorldData
                    >
                >::Iter
            }
        })
        .fold(quote! { ::std::iter::Empty<F> }, |chain, ty| {
            quote! {
                ::std::iter::Chain<#chain, #ty>
            }
        });

    // TODO: Allow generic enum derive(Entity). Should be possible?
    let lifetime: syn::Lifetime = syn::parse_str("'__stecs__f").unwrap();
    let type_param: syn::TypeParam = syn::parse_str("__stecs__F").unwrap();

    // TODO: We could probably remove this requirement.
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            ident,
            "derive(Entity) for enums must not have any generics",
        ));
    }

    /*
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let generics_with_lifetime = generics_with_new_lifetime(&input.generics, &lifetime);
    let (impl_generics_with_lifetime, ty_generics_with_lifetime, where_clause_with_lifetime) =
        generics_with_lifetime.split_for_impl();

    let generics_with_type_param = generics_with_new_type_param(&input.generics, &type_param);
    let (impl_generics_with_type_param, ty_generics_with_type_param, where_clause_with_type_param) =
        generics_with_type_param.split_for_impl();
    */

    Ok(quote! {
        // Id

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
        #vis enum #ident_id {
            #(
                #variant_idents(<#variant_tys as ::stecs::Entity>::Id),
            )*
        }

        // IdFetch

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
            type Item<'f> = ::stecs::EntityId<#ident> where Self: 'f;

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

            unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
            where
                Self: 'f,
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

            fn filter_by_outer<__stecs__DOuter: ::stecs::world::WorldData>(
                fetch: &mut Option<Self>,
            ) {
                if ::std::any::TypeId::of::<__stecs__DOuter>() !=
                    ::std::any::TypeId::of::<#ident_world_data>() {
                    *fetch = ::std::option::Option::None;
                }
            }
        }

        // RefFetch

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
            type Item<'f> = #ident_ref<'f> where Self: 'f;

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

            unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
            where
                Self: 'f,
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

            fn filter_by_outer<__stecs__DOuter: ::stecs::world::WorldData>(
                fetch: &mut Option<Self>,
            ) {
                if ::std::any::TypeId::of::<__stecs__DOuter>() !=
                    ::std::any::TypeId::of::<#ident_world_data>() {
                    *fetch = ::std::option::Option::None;
                }
            }
        }

        impl<'q> ::stecs::Query for #ident_ref<'q> {
            type Fetch<'w> = #ident_ref_fetch<'w>;
        }

        impl<'q> ::stecs::QueryShared for #ident_ref<'q> {
        }

        // Ref

        #[derive(::std::clone::Clone)]
        #vis enum #ident_ref<#lifetime> {
            #(
                #variant_idents(<#variant_tys as ::stecs::Entity>::Ref<#lifetime>),
            )*
        }

        // RefFetch

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
            type Item<'f> = #ident_ref_mut<'f> where Self: 'f;

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

            unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
            where
                Self: 'f,
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

            fn filter_by_outer<__stecs__DOuter: ::stecs::world::WorldData>(
                fetch: &mut Option<Self>,
            ) {
                if ::std::any::TypeId::of::<__stecs__DOuter>() !=
                    ::std::any::TypeId::of::<#ident_world_data>() {
                    *fetch = ::std::option::Option::None;
                }
            }
        }

        impl<'q> ::stecs::Query for #ident_ref_mut<'q> {
            type Fetch<'w> = #ident_ref_mut_fetch<'w>;
        }

        // RefMut

        #vis enum #ident_ref_mut<#lifetime> {
            #(
                #variant_idents(<#variant_tys as ::stecs::Entity>::RefMut<#lifetime>),
            )*
        }

        // WorldFetch

        #[allow(non_snake_case)]
        #vis struct #ident_world_fetch<#lifetime, #type_param>
        where
            #type_param: ::stecs::query::fetch::Fetch + #lifetime,
        {
            #(
                #variant_idents:
                    <
                        <
                            #variant_tys
                            as ::stecs::Entity
                        >::WorldData
                        as ::stecs::world::WorldData
                    >::Fetch<#lifetime, #type_param>,
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

        // TODO: Clean up generic names if we want to allow generic enums.
        impl<'w, F> ::stecs::world::WorldFetch<'w, #ident_world_data> for #ident_world_fetch<'w, F>
        where
            F: ::stecs::query::fetch::Fetch,
        {
            type Fetch = F;

            type Iter = #world_fetch_iter;

            unsafe fn get<'f>(&self, id: #ident_id) -> ::std::option::Option<F::Item<'f>> {
                match id {
                    #(
                        #ident_id::#variant_idents(id) => {
                            type WorldData = <#variant_tys as ::stecs::Entity>::WorldData;

                            // Safety: TODO
                            unsafe {
                                <
                                    <WorldData as ::stecs::world::WorldData>::Fetch<'w, F>
                                    as ::stecs::world::WorldFetch<WorldData>
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
                            <
                                <
                                    #variant_tys
                                    as ::stecs::Entity
                                >::WorldData
                                as ::stecs::world::WorldData
                            >::Fetch<'w, F>
                            as ::stecs::world::WorldFetch<
                                <#variant_tys as ::stecs::Entity>::WorldData
                            >
                        >
                        ::iter(&mut self.#variant_idents)
                    );
                )*

                iter
            }
        }

        // WorldData

        // TODO: Consider exposing the `WorldData` struct. In this case, convert
        // field names to snake case first.
        #[allow(non_snake_case)]
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

            /*
            fn entity<E>(&self, id: ::stecs::EntityId<E>) -> ::std::option::Option<E::Ref<'_>>
            where
                E: ::stecs::entity::EntityVariant<#ident>,
            {
                /*match id.get() {
                    #(
                        #ident_id::#variant_idents(id) => {
                            let id = ::stecs::EntityId::new(id);
                            #ident_ref::#variant_idents(self.#variant_idents.entity(id))
                        }
                    )*
                }*/

                todo!()
            }
            */

            fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
            where
                F: ::stecs::query::fetch::Fetch + 'w,
            {
                let mut fetch = #ident_world_fetch {
                    #(
                        #variant_idents:
                            <<#variant_tys as ::stecs::Entity>::WorldData as ::stecs::world::WorldData>
                            ::fetch::<F>(&self.#variant_idents),
                    )*
                };

                /*#(
                    <
                        <
                            <
                                #variant_tys
                                as ::stecs::Entity
                            >::WorldData
                            as ::stecs::world::WorldData
                        >::Fetch<'w, F>
                        as ::stecs::world::WorldFetch<
                            'w,
                            <
                                #variant_tys
                                as ::stecs::Entity
                            >::WorldData
                        >
                    >
                    ::filter_by_outer::<Self>(&mut fetch.#variant_idents);
                )**/

                fetch
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

        // Entity

        impl ::stecs::Entity for #ident {
            type Id = #ident_id;
            type Ref<#lifetime> = #ident_ref<#lifetime>;
            type RefMut<#lifetime> = #ident_ref_mut<#lifetime>;
            type Fetch<'__stecs__w> = #ident_ref_fetch<'__stecs__w>;
            type FetchMut<'__stecs__w> = #ident_ref_mut_fetch<'__stecs__w>;
            type FetchId<'__stecs__w> = #ident_id_fetch<'__stecs__w>;
            type WorldData = #ident_world_data;

            fn from_ref<'f>(entity: Self::Ref<'f>) -> Self {
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
    })
}

fn add_additional_bounds_to_generic_params(generics: &mut syn::Generics) {
    for type_param in generics.type_params_mut() {
        type_param
            .bounds
            .push(syn::TypeParamBound::Trait(syn::TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: syn::parse_quote!(::stecs::Component),
            }))
    }
}
