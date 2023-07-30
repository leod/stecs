use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error, Result};

use crate::utils::{members_as_idents, struct_fields};

pub fn derive(input: DeriveInput) -> Result<TokenStream2> {
    let ident = &input.ident;
    let vis = &input.vis;
    let data = match input.data {
        syn::Data::Struct(ref data) => Ok(data),
        _ => Err(Error::new_spanned(
            ident,
            "derive(Query) only supports structs",
        )),
    }?;

    let ident_fetch = syn::Ident::new(&format!("{ident}StecsInternalFetch"), ident.span());

    let (field_tys, field_members) = struct_fields(&data.fields);
    let field_idents = members_as_idents(&field_members);

    let lifetime = input
        .generics
        .lifetimes()
        .next()
        .map(|x| x.lifetime.clone());
    let lifetime = match lifetime {
        Some(x) => x,
        None => {
            return Err(Error::new_spanned(
                input.generics,
                "must have exactly one lifetime parameter",
            ))
        }
    };

    // TODO: We could probably remove this requirement.
    if input.generics.params.len() != 1 {
        return Err(Error::new_spanned(
            ident,
            "derive(Query) must have exactly one lifetime parameter and no type parameters",
        ));
    }

    Ok(quote! {
        // Fetch

        #[allow(unused, non_snake_case)]
        #[derive(::std::clone::Clone, ::std::marker::Copy)]
        #vis struct #ident_fetch<#lifetime> {
            __stecs__len: usize,
            #(
                #field_idents: <#field_tys as ::stecs::Query>::Fetch<#lifetime>,
            )*
        }

        unsafe impl<#lifetime> ::stecs::query::fetch::Fetch
        for #ident_fetch<#lifetime> {
            type Item<'__stecs__f> = #ident<'__stecs__f> where Self: '__stecs__f;

            fn new<__stecs__T: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &__stecs__T,
            ) -> ::std::option::Option<Self> {
                #(
                    let #field_idents = <#field_tys as ::stecs::Query>::Fetch::<#lifetime>::new(
                        ids,
                        columns,
                    )?;
                )*

                ::std::option::Option::Some(#ident_fetch {
                    __stecs__len: ids.len(),
                    #(
                        #field_idents,
                    )*
                })
            }

            fn len(&self) -> usize {
                self.__stecs__len
            }

            unsafe fn get<'__stecs__f>(&self, index: usize) -> Self::Item<'__stecs__f>
            where
                Self: '__stecs__f,
            {
                #ident {
                    #(
                        #field_idents:
                            ::stecs::query::fetch::Fetch::get(&self.#field_idents, index),
                    )*
                }
            }

            fn check_borrows(checker: &mut ::stecs::query::borrow_checker::BorrowChecker) {
                #(
                    <#field_tys as ::stecs::Query>::Fetch::<#lifetime>::check_borrows(checker);
                )*
            }
        }

        // Query

        impl<'__stecs__q> ::stecs::Query for #ident<'__stecs__q> {
            type Fetch<#lifetime> = #ident_fetch<#lifetime>;
        }
    })
}
