use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DataStruct, DeriveInput, Result};

use crate::utils::{
    associated_ident, generics_with_new_lifetime, members_as_idents, struct_fields,
};

pub fn derive(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream2> {
    let ident = &input.ident;
    let vis = &input.vis;

    let ident_columns = associated_ident(ident, "Columns");
    let ident_ref = associated_ident(ident, "Ref");
    let ident_ref_fetch = associated_ident(ident, "RefFetch");
    let ident_ref_mut = associated_ident(ident, "RefMut");
    let ident_ref_mut_fetch = associated_ident(ident, "RefMutFetch");

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
        #[allow(unused, non_camel_case_types)]
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

        #[allow(unused, non_snake_case, non_camel_case_types)]
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
        #[allow(unused, non_snake_case, non_camel_case_types)]
        #[derive(::std::clone::Clone)]
        #vis struct #ident_ref #impl_generics_with_lifetime #where_clause_with_lifetime {
            #(
                #vis #field_idents: &#lifetime #field_tys,
            )*
            __stecs__phantom: ::std::marker::PhantomData<&#lifetime ()>,
        }

        // RefMutFetch

        #[allow(unused, non_snake_case, non_camel_case_types)]
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
        #[allow(unused, non_snake_case, non_camel_case_types)]
        #vis struct #ident_ref_mut #impl_generics_with_lifetime #where_clause_with_lifetime {
            #(
                #vis #field_idents: &#lifetime mut #field_tys,
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
