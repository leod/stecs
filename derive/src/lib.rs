use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod archetype_set;
mod entity;
mod utils;

#[proc_macro_derive(ArchetypeSet)]
pub fn derive_archetype_set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match archetype_set::derive(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_derive(Entity)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match entity::derive(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}
