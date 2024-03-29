use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod entity;
mod query;
mod query_shared;
mod utils;

#[proc_macro_derive(Entity, attributes(stecs))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match entity::derive(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_derive(Query)]
pub fn derive_query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match query::derive(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_derive(QueryShared)]
pub fn derive_query_shared(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match query_shared::derive(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}
