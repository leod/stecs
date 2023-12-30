use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error, Result};

use crate::utils::{members_as_idents, struct_fields};

pub fn derive(input: DeriveInput) -> Result<TokenStream2> {
    let ident = &input.ident;
    let data = match input.data {
        syn::Data::Struct(ref data) => Ok(data),
        _ => Err(Error::new_spanned(
            ident,
            "derive(QueryShared) only supports structs",
        )),
    }?;

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
            "derive(QueryShared) must have exactly one lifetime parameter and no type parameters",
        ));
    }

    Ok(quote! {
        // QueryShared

        unsafe impl<'__stecs__q> ::stecs::QueryShared for #ident<'__stecs__q>
        where
            #(for<#lifetime> #field_tys: ::stecs::QueryShared,)*
        {
        }

        const _: fn (#ident) = |query: #ident| {
            fn check_field<Q: ::stecs::QueryShared>(_: Q) {}

            #(check_field(query.#field_idents);)*
        };
    })
}
