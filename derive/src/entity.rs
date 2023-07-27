use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_quote, DataEnum, DataStruct, DeriveInput, Error, Result};

use crate::utils::{
    generics_with_new_lifetime, generics_with_new_type_param, member_as_idents, struct_fields,
};

pub fn derive(input: DeriveInput) -> Result<TokenStream2> {
    match input.data {
        syn::Data::Struct(ref data) => derive_struct(&input, data),
        syn::Data::Enum(ref data) => derive_enum(&input, data),
        _ => {
            return Err(Error::new_spanned(
                input.ident,
                "derive(Entity) only supports structs and enums",
            ))
        }
    }
}

fn derive_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream2> {
    let ident = &input.ident;
    let vis = &input.vis;

    let ident_columns = syn::Ident::new(&format!("{}StecsInternalColumns", ident), ident.span());
    let ident_ref = syn::Ident::new(&format!("{}StecsInternalRef", ident), ident.span());
    let ident_ref_fetch = syn::Ident::new(&format!("{}StecsInternalRefFetch", ident), ident.span());
    let ident_ref_mut = syn::Ident::new(&format!("{}StecsInternalRefMut", ident), ident.span());
    let ident_ref_mut_fetch =
        syn::Ident::new(&format!("{}StecsInternalRefMutFetch", ident), ident.span());

    let (field_tys, field_members) = struct_fields(&data.fields);
    let field_idents = member_as_idents(&field_members);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetime: syn::Lifetime = syn::parse_str("'__stecs__f").unwrap();
    let generics_with_lifetime = generics_with_new_lifetime(&input.generics, &lifetime);
    let (impl_generics_with_lifetime, ty_generics_with_lifetime, where_clause_with_lifetime) =
        generics_with_lifetime.split_for_impl();

    Ok(quote! {
        // Columns

        // TODO: Provide a way to derive traits for the column struct.
        // Otherwise, we lose the ability to derive things for our World.
        #[allow(unused)]
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

            type Fetch<'__stecs__w> = #ident_ref_fetch #ty_generics;

            type FetchMut<'__stecs__w> = #ident_ref_mut_fetch #ty_generics;

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

                None
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
            ) -> Self::Fetch<'__stecs__w>
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
            ) -> Self::FetchMut<'__stecs__w>
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
                ids: &::stecs::column::Column<thunderdome::Index>,
                columns: &__stecs__A,
            ) -> ::std::option::Option<Self>
            {
                if ::std::any::TypeId::of::<__stecs__A>() ==
                       ::std::any::TypeId::of::<#ident_columns #ty_generics>() {
                    let columns: &#ident_columns #ty_generics =
                        (columns as &dyn ::std::any::Any).downcast_ref().unwrap();

                    Some(
                        <#ident_columns #ty_generics as ::stecs::entity::Columns>
                            ::new_fetch(columns, ids.len()),
                    )
                } else {
                    None
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

        // Ref

        // FIXME: This should be a tuple struct for tuple structs.
        #[allow(unused, non_snake_case)]
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
                ids: &::stecs::column::Column<thunderdome::Index>,
                columns: &__stecs__A,
            ) -> ::std::option::Option<Self>
            {
                if ::std::any::TypeId::of::<__stecs__A>() ==
                       ::std::any::TypeId::of::<#ident_columns #ty_generics>() {
                    let columns: &#ident_columns #ty_generics =
                        (columns as &dyn ::std::any::Any).downcast_ref().unwrap();

                    Some(
                        <#ident_columns #ty_generics as ::stecs::entity::Columns>
                            ::new_fetch_mut(columns, ids.len()),
                    )
                } else {
                    None
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

        // ConcreteEntity

        impl #impl_generics ::stecs::entity::ConcreteEntity for #ident #ty_generics #where_clause {
            type Columns = #ident_columns #ty_generics;
        }

        // Entity

        impl #impl_generics ::stecs::Entity for #ident #ty_generics #where_clause {
            type Id = ::stecs::archetype::EntityKey<Self>;

            type Ref<#lifetime> = #ident_ref #ty_generics_with_lifetime;

            type RefMut<#lifetime> = #ident_ref_mut #ty_generics_with_lifetime;

            type WorldData = ::stecs::archetype::Archetype<#ident_columns #ty_generics>;
        }
    })
}

fn derive_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream2> {
    let ident = &input.ident;
    let vis = &input.vis;

    let ident_id = syn::Ident::new(&format!("{}StecsInternalId", ident), ident.span());
    let ident_ref = syn::Ident::new(&format!("{}StecsInternalRef", ident), ident.span());
    let ident_ref_mut = syn::Ident::new(&format!("{}StecsInternalRefMut", ident), ident.span());
    let ident_world_fetch =
        syn::Ident::new(&format!("{}StecsInternalWorldFetch", ident), ident.span());
    let ident_world_data =
        syn::Ident::new(&format!("{}StecsInternalWorldData", ident), ident.span());

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
        )]
        #vis enum #ident_id {
            #(
                #variant_idents(<#variant_tys as ::stecs::Entity>::Id),
            )*
        }

        // Ref

        #vis enum #ident_ref<#lifetime> {
            #(
                #variant_idents(&#lifetime #variant_tys),
            )*
        }

        // RefMut

        #vis enum #ident_ref_mut<#lifetime> {
            #(
                #variant_idents(&#lifetime mut #variant_tys),
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

        #vis impl<#lifetime, #type_param> ::std::clone::Clone
        for #ident_world_fetch<#lifetime, #type_param>
        where
            #type_param: ::stecs::query::fetch::Fetch + #lifetime,
        {
            fn clone(&self) -> Self {
                *self
            }
        }

        #vis impl<#lifetime, #type_param> ::std::marker::Copy
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

            unsafe fn get<'f>(&self, id: #ident_id) -> Option<F::Item<'f>>
            where
                Self: 'f,
            {
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
                            <<#variant_tys as ::stecs::Entity>::WorldData as ::stecs::world::WorldData>::Fetch<'w, F>
                            as ::stecs::world::WorldFetch<<#variant_tys as ::stecs::Entity>::WorldData>
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
        #[derive(::std::default::Default)]
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

                assert_eq!(::std::any::TypeId::of::<E>(), ::std::any::TypeId::of::<#ident_id>());

                let id: #ident_id =
                    match <E as ::stecs::entity::EntityVariant<#ident>>::into_outer(entity) {
                        #(
                            #ident::#variant_idents(entity) => {
                                #ident_id::#variant_idents(self.#variant_idents.spawn(entity).get())
                            }
                        )*
                    };

                let id = ::stecs::EntityId::<#ident>::new_unchecked(id);

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
                            let id = ::stecs::EntityId::<#variant_tys>::new_unchecked(id);
                            self.#variant_idents.despawn(id).map(|entity| #ident::#variant_idents(entity))
                        }
                    )*
                }
            }

            fn entity<E>(&self, id: ::stecs::EntityId<E>) -> ::std::option::Option<E::Ref<'_>>
            where
                E: ::stecs::entity::EntityVariant<#ident>,
            {
                /*match id.get() {
                    #(
                        #ident_id::#variant_idents(id) => {
                            let id = ::stecs::EntityId::new_unchecked(id);
                            #ident_ref::#variant_idents(self.#variant_idents.entity(id))
                        }
                    )*
                }*/

                todo!()
            }

            fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
            where
                F: ::stecs::query::fetch::Fetch + 'w,
            {
                #ident_world_fetch {
                    #(
                        #variant_idents:
                            <<#variant_tys as ::stecs::Entity>::WorldData as ::stecs::world::WorldData>
                            ::fetch::<F>(&self.#variant_idents),
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

        // Entity

        impl ::stecs::Entity for #ident {
            type Id = #ident_id;
            type Ref<#lifetime> = #ident_ref<#lifetime>;
            type RefMut<#lifetime> = #ident_ref_mut<#lifetime>;
            type WorldData = #ident_world_data;
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
