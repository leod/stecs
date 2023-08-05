mod r#enum;
mod r#struct;

use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, Error, Result};

pub fn derive(input: DeriveInput) -> Result<TokenStream2> {
    match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => r#struct::derive(&input, fields),
        syn::Data::Enum(ref data) => r#enum::derive(&input, data),
        _ => Err(Error::new_spanned(
            input.ident,
            "derive(Entity) only supports structs and enums",
        )),
    }
}
