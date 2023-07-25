use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error, Result};

use crate::utils::{
    generics_with_new_lifetime, generics_with_new_type_param, member_as_idents, struct_fields,
};

pub fn derive(mut input: DeriveInput) -> Result<TokenStream2> {
    let ident = input.ident;
    let data = match input.data {
        syn::Data::Struct(s) => s,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "derive(Entity) does not support enums or unions",
            ))
        }
    };

    let ident_columns = syn::Ident::new(&format!("{}StecsInternalColumns", ident), ident.span());
    let ident_ref = syn::Ident::new(&format!("{}StecsInternalRef", ident), ident.span());
    let ident_ref_fetch = syn::Ident::new(&format!("{}StecsInternalRefFetch", ident), ident.span());
    let ident_ref_mut = syn::Ident::new(&format!("{}StecsInternalRefMut", ident), ident.span());
    let ident_ref_mut_fetch =
        syn::Ident::new(&format!("{}StecsInternalRefMutFetch", ident), ident.span());

    let (field_tys, field_members) = struct_fields(&data.fields);
    let field_idents = member_as_idents(&field_members);

    add_additional_bounds_to_generic_params(&mut input.generics);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetime: syn::Lifetime = syn::parse_str("'__stecs__f").unwrap();
    let generics_with_lifetime = generics_with_new_lifetime(&input.generics, &lifetime);
    let (impl_generics_with_lifetime, ty_generics_with_lifetime, where_clause_with_lifetime) =
        generics_with_lifetime.split_for_impl();

    let set_param: syn::TypeParam = syn::parse_str("__stecs__S: ::stecs::ArchetypeSet").unwrap();
    let generics_with_set = generics_with_new_type_param(&input.generics, &set_param);
    let (impl_generics_with_set, ty_generics_with_set, where_clause_with_set) =
        generics_with_set.split_for_impl();

    Ok(quote! {
        // Columns

        // TODO: Provide a way to derive traits for the column struct.
        // Otherwise, we lose the ability to derive things for our World.
        #[allow(unused)]
        struct #ident_columns #impl_generics #where_clause {
            #(
                #field_idents: ::std::cell::RefCell<::stecs::internal::Column<#field_tys>>
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
            -> ::std::option::Option<&::std::cell::RefCell<::stecs::internal::Column<__stecs__C>>>
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
        }

        // RefFetch

        #[allow(unused)]
        struct #ident_ref_fetch #impl_generics #where_clause {
            __stecs__len: usize,
            #(
                #field_idents: ::stecs::internal::ColumnRawParts<#field_tys>
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

        unsafe impl #impl_generics_with_set ::stecs::query::fetch::FetchFromSet<__stecs__S>
        for #ident_ref_fetch #ty_generics
        where
            __stecs__S: ::stecs::ArchetypeSet,
        {
            fn new<E: ::stecs::archetype_set::InArchetypeSet<__stecs__S>>(
                ids: &::stecs::internal::Column<thunderdome::Index>,
                columns: &E::Columns,
            ) -> ::std::option::Option<Self>
            {
                if ::std::any::TypeId::of::<E>() ==
                       ::std::any::TypeId::of::<#ident #ty_generics>() {
                    let columns: &#ident_columns #ty_generics =
                        (columns as &dyn ::std::any::Any).downcast_ref().unwrap();

                    Some(
                        <#ident_ref #ty_generics as ::stecs::entity::EntityBorrow<'_>>
                            ::new_fetch(ids.len(), columns),
                    )
                } else {
                    None
                }
            }
        }

        // Ref

        #[allow(unused)]
        struct #ident_ref #impl_generics_with_lifetime #where_clause_with_lifetime {
            #(
                #field_idents: &#lifetime #field_tys,
            )*
            __stecs__phantom: ::std::marker::PhantomData<&#lifetime ()>,
        }

        impl #impl_generics_with_lifetime ::stecs::entity::EntityBorrow<#lifetime>
        for #ident_ref #ty_generics_with_lifetime #where_clause_with_lifetime {
            type Entity = #ident #ty_generics;

            type Fetch<'__stecs_w> = #ident_ref_fetch #ty_generics where '__stecs_w: #lifetime;

            fn new_fetch<'__stecs_w>(
                len: usize,
                columns: &'__stecs_w <Self::Entity as ::stecs::Entity>::Columns,
            ) -> Self::Fetch<'__stecs_w>
            where
                '__stecs_w: #lifetime,
            {
                #(
                    ::std::debug_assert_eq!(len, columns.#field_idents.borrow().len());
                )*

                #ident_ref_fetch {
                    __stecs__len: len,
                    #(
                        #field_idents: columns.#field_idents.borrow().as_raw_parts()
                    ),*
                }
            }
        }

        // RefMutFetch

        #[allow(unused)]
        struct #ident_ref_mut_fetch #impl_generics #where_clause {
            __stecs__len: usize,
            #(
                #field_idents: ::stecs::internal::ColumnRawPartsMut<#field_tys>
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

        unsafe impl #impl_generics_with_set ::stecs::query::fetch::FetchFromSet<__stecs__S>
        for #ident_ref_mut_fetch #ty_generics
        where
            __stecs__S: ::stecs::ArchetypeSet,
        {
            fn new<E: ::stecs::archetype_set::InArchetypeSet<__stecs__S>>(
                ids: &::stecs::internal::Column<thunderdome::Index>,
                columns: &E::Columns,
            ) -> ::std::option::Option<Self>
            {
                if ::std::any::TypeId::of::<E>() ==
                       ::std::any::TypeId::of::<#ident #ty_generics>() {
                    let columns: &#ident_columns #ty_generics =
                        (columns as &dyn ::std::any::Any).downcast_ref().unwrap();

                    Some(
                        <#ident_ref_mut #ty_generics as ::stecs::entity::EntityBorrow<'_>>
                            ::new_fetch(ids.len(), columns),
                    )
                } else {
                    None
                }
            }
        }

        // RefMut

        #[allow(unused)]
        struct #ident_ref_mut #impl_generics_with_lifetime #where_clause_with_lifetime {
            #(
                #field_idents: &#lifetime mut #field_tys,
            )*
            __stecs__phantom: ::std::marker::PhantomData<&#lifetime mut ()>,
        }

        impl #impl_generics_with_lifetime ::stecs::entity::EntityBorrow<#lifetime>
        for #ident_ref_mut #ty_generics_with_lifetime #where_clause_with_lifetime {
            type Entity = #ident #ty_generics;

            type Fetch<'__stecs_w> = #ident_ref_mut_fetch #ty_generics where '__stecs_w: #lifetime;

            fn new_fetch<'__stecs_w>(
                len: usize,
                columns: &'__stecs_w #ident_columns #ty_generics,
            ) -> Self::Fetch<'__stecs_w>
            where
                '__stecs_w: #lifetime,
            {
                #(
                    ::std::debug_assert_eq!(len, columns.#field_idents.borrow().len());
                )*

                #ident_ref_mut_fetch {
                    __stecs__len: len,
                    #(
                        #field_idents: columns.#field_idents.borrow_mut().as_raw_parts_mut(),
                    )*
                }
            }
        }

        // Entity

        impl #impl_generics ::stecs::Entity for #ident #ty_generics #where_clause {
            type Columns = #ident_columns #ty_generics;

            type Borrow<#lifetime> = #ident_ref #ty_generics_with_lifetime;

            type BorrowMut<#lifetime> = #ident_ref_mut #ty_generics_with_lifetime;
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
