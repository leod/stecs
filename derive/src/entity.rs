mod r#enum;
mod r#struct;

use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, Error, Result};

use crate::utils::parse_attr_names;

pub fn derive(input: DeriveInput) -> Result<TokenStream2> {
    let attr_names = parse_attr_names(&input.attrs)?;

    match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => r#struct::derive(&input, fields),
        syn::Data::Enum(ref data) => r#enum::derive(&input, data, attr_names),
        _ => Err(Error::new_spanned(
            input.ident,
            "derive(Entity) only supports structs and enums",
        )),
    }
}
