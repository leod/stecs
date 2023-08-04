mod r#enum;
mod r#struct;

use proc_macro2::TokenStream as TokenStream2;
use syn::{parse::Parse, DeriveInput, Error, Result};

use crate::utils::parse_attr_names;

pub fn derive(input: DeriveInput) -> Result<TokenStream2> {
    let attr_names = parse_attr_names(&input.attrs)?;

    match input.data {
        syn::Data::Struct(ref data) => r#struct::derive(&input, data),
        syn::Data::Enum(ref data) => r#enum::derive(&input, data, attr_names),
        _ => Err(Error::new_spanned(
            input.ident,
            "derive(Entity) only supports structs and enums",
        )),
    }
}
