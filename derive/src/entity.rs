mod r#enum;
mod r#struct;

use proc_macro2::TokenStream as TokenStream2;
use syn::{parse::Parse, DeriveInput, Error, Result};

struct AttrNames {
    names: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]>,
}

impl Parse for AttrNames {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let names = input.parse_terminated(syn::Ident::parse, syn::Token![,])?;

        Ok(Self { names })
    }
}

pub fn derive(input: DeriveInput) -> Result<TokenStream2> {
    let mut attrs = Vec::new();

    for attr in &input.attrs {
        if !attr.path().is_ident("stecs") {
            continue;
        }

        let attr_names: AttrNames = attr.parse_args()?;
        attrs.extend(attr_names.names.into_iter().map(|ident| ident.to_string()));
    }

    match input.data {
        syn::Data::Struct(ref data) => r#struct::derive(&input, data),
        syn::Data::Enum(ref data) => r#enum::derive(&input, data, attrs),
        _ => Err(Error::new_spanned(
            input.ident,
            "derive(Entity) only supports structs and enums",
        )),
    }
}
