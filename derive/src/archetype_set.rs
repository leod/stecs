use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Error, Result};

use crate::utils::{generics_with_new_lifetime, member_as_idents, struct_fields};

pub fn derive(mut input: DeriveInput) -> Result<TokenStream2> {
    let ident = input.ident;
    let vis = input.vis;
    let data = match input.data {
        syn::Data::Struct(s) => s,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "derive(ArchetypeSet) does not support enums or unions",
            ))
        }
    };

    let ident_any_entity_id =
        syn::Ident::new(&format!("{}StecsInternalAnyEntityId", ident), ident.span());
    let ident_any_entity =
        syn::Ident::new(&format!("{}StecsInternalAnyEntity", ident), ident.span());
    let ident_fetch = syn::Ident::new(&format!("{}StecsInternalFetch", ident), ident.span());

    let (field_tys, field_members) = struct_fields(&data.fields);
    let field_idents = member_as_idents(&field_members);
    let field_idents_upper_camel: Vec<_> = field_idents
        .iter()
        .map(|ident| {
            // TODO: This can certainly be done better.
            Ident::new(
                &ident
                    .clone()
                    .into_owned()
                    .to_string()
                    .to_case(Case::UpperCamel),
                ident.span(),
            )
        })
        .collect();

    /*
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetime: syn::Lifetime = syn::parse_str("'__stecs__w").unwrap();
    let generics_with_lifetime = generics_with_new_lifetime(&input.generics, &lifetime);
    let (impl_generics_with_lifetime, ty_generics_with_lifetime, where_clause_with_lifetime) =
        generics_with_lifetime.split_for_impl();
    */

    Ok(quote! {
        // TODO: Eq, PartialOrd, Ord, Hash
        // TODO: We'd need a `PhantomData` in this enum to really support
        // generic worlds. I don't think we need generic worlds, but we should
        // provide the user with a useful error message.
        #[derive(
            ::std::clone::Clone,
            ::std::marker::Copy,
            ::std::fmt::Debug,
            ::std::cmp::PartialEq,
        )]
        #vis enum #ident_any_entity_id {
            #(
                #field_idents_upper_camel(<#field_tys as ::stecs::ArchetypeSet>::AnyEntityId),
            )*
        }

        #vis enum #ident_any_entity {
            #(
                #field_idents_upper_camel(<#field_tys as ::stecs::ArchetypeSet>::AnyEntity),
            )*
        }

        /*#[derive(::std::clone::Clone)]
        #vis struct #ident_fetch<'__stecs__w, __stecs__F>
        where
            __stecs__F: ::stecs::query::fetch::FetchFromSet<#ident> + '__stecs__w
        {
            #(
                #field_idents: <#field_tys as ::stecs::ArchetypeSet>::Fetch<'__stecs__w, __stecs__F>,
            )*
        }*/
    })
}
