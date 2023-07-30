use std::borrow::Cow;

use proc_macro2::Span;

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
