use std::borrow::Cow;

use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Result};

pub fn associated_ident(ident: &syn::Ident, ty: &str) -> syn::Ident {
    syn::Ident::new(&format!("__stecs__{ident}{ty}"), ident.span())
}

pub fn parse_attr_names(attrs: &[syn::Attribute]) -> Result<Vec<String>> {
    let mut names = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("stecs") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident() {
                names.push(ident.to_string());
            }

            Ok(())
        })?;
    }

    Ok(names)
}

pub struct Derives {
    pub id_derives: TokenStream2,
    pub world_data_derives: TokenStream2,
    pub columns_derives: TokenStream2,
}

pub fn get_attr_derives(attrs: &[syn::Attribute]) -> Result<Derives> {
    let mut id_paths = Vec::new();
    let mut world_data_paths = Vec::new();
    let mut columns_paths = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("stecs") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            let paths = if meta.path.is_ident("derive_id") {
                &mut id_paths
            } else if meta.path.is_ident("derive_world_data") {
                &mut world_data_paths
            } else if meta.path.is_ident("derive_columns") {
                &mut columns_paths
            } else {
                return Err(syn::Error::new(attr.span(), "Unknown attribute"));
            };

            let content;
            syn::parenthesized!(content in meta.input);

            paths.extend(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated(
                    &content,
                )?,
            );

            Ok(())
        })?;
    }

    Ok(Derives {
        id_derives: quote! { #[derive(#(#id_paths,)*)] },
        world_data_derives: quote! { #[derive(#(#world_data_paths,)*)] },
        columns_derives: quote! { #[derive(#(#columns_paths,)*)] },
    })
}

// Copied from `hecs`.
pub fn struct_fields(fields: &syn::Fields) -> (Vec<&syn::Type>, Vec<syn::Member>) {
    match fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| (&f.ty, syn::Member::Named(f.ident.clone().unwrap())))
            .unzip(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                (
                    &f.ty,
                    syn::Member::Unnamed(syn::Index {
                        index: i as u32,
                        span: Span::call_site(),
                    }),
                )
            })
            .unzip(),
        syn::Fields::Unit => (Vec::new(), Vec::new()),
    }
}

// Copied from `hecs`.
pub fn members_as_idents(members: &[syn::Member]) -> Vec<Cow<'_, syn::Ident>> {
    members
        .iter()
        .map(|member| match member {
            syn::Member::Named(ident) => Cow::Borrowed(ident),
            &syn::Member::Unnamed(syn::Index { index, span }) => {
                Cow::Owned(syn::Ident::new(&format!("tuple_field_{index}"), span))
            }
        })
        .collect()
}

pub fn generics_with_new_lifetime(
    generics: &syn::Generics,
    lifetime: &syn::Lifetime,
) -> syn::Generics {
    let mut new_generics = generics.clone();

    let lifetime_param = syn::LifetimeParam::new(lifetime.clone());

    let mut new_params: Vec<syn::GenericParam> = Vec::new();
    new_params.push(syn::GenericParam::Lifetime(lifetime_param));
    new_params.extend(new_generics.params);

    new_generics.params = syn::punctuated::Punctuated::from_iter(new_params);
    new_generics
}

pub fn generics_with_new_type_param(
    generics: &syn::Generics,
    type_param: &syn::TypeParam,
) -> syn::Generics {
    let mut new_generics = generics.clone();

    new_generics
        .params
        .push(syn::GenericParam::Type(type_param.clone()));

    new_generics
}
