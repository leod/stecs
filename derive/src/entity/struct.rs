use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Result};

use crate::utils::{
    associated_ident, generics_with_new_lifetime, get_attr_derives, parse_attr_names, Derives,
};

#[derive(Default)]
struct Fields<'a> {
    idents: Vec<&'a syn::Ident>,
    tys: Vec<&'a syn::Type>,
}

impl<'a> FromIterator<&'a syn::Field> for Fields<'a> {
    fn from_iter<T: IntoIterator<Item = &'a syn::Field>>(iter: T) -> Self {
        let mut result = Fields::default();

        for field in iter {
            result
                .idents
                .push(field.ident.as_ref().expect("Expected named struct"));
            result.tys.push(&field.ty);
        }

        result
    }
}

fn split_fields(fields: &syn::FieldsNamed) -> Result<(Fields, Fields)> {
    let (mut field_comps, mut field_flats) = (Vec::new(), Vec::new());

    for field in &fields.named {
        let attrs = parse_attr_names(&field.attrs)?;

        if attrs.iter().any(|a| a == "flat") {
            &mut field_flats
        } else {
            &mut field_comps
        }
        .push(field);
    }

    Ok((
        field_comps.into_iter().collect(),
        field_flats.into_iter().collect(),
    ))
}

pub fn derive(input: &DeriveInput, fields: &syn::FieldsNamed) -> Result<TokenStream2> {
    let ident = &input.ident;
    let vis = &input.vis;

    let ident_columns = associated_ident(ident, "Columns");
    let ident_ref = associated_ident(ident, "Ref");
    let ident_ref_mut = associated_ident(ident, "RefMut");
    let ident_ref_fetch = associated_ident(ident, "RefFetch");
    let ident_ref_mut_fetch = associated_ident(ident, "RefMutFetch");

    let Derives {
        columns_derives,
        ref_derives,
        ..
    } = get_attr_derives(&input.attrs)?;

    let (
        Fields {
            idents: field_comp_idents,
            tys: field_comp_tys,
        },
        Fields {
            idents: field_flat_idents,
            tys: field_flat_tys,
        },
    ) = split_fields(fields)?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let where_clause_predicates = where_clause.map(|where_clause| &where_clause.predicates);

    let lifetime: syn::Lifetime = syn::parse_str("'__stecs__a").unwrap();
    let lifetime2: syn::Lifetime = syn::parse_str("'__stecs__b").unwrap();

    let generics_lifetime = generics_with_new_lifetime(&input.generics, &lifetime);
    let (impl_generics_lifetime, ty_generics_lifetime, where_clause_lifetime) =
        generics_lifetime.split_for_impl();

    let generics_lifetime2 = generics_with_new_lifetime(&input.generics, &lifetime2);
    let (_, ty_generics_lifetime2, _) = generics_lifetime2.split_for_impl();

    Ok(quote! {
        // Entity

        impl #impl_generics ::stecs::Entity for #ident #ty_generics #where_clause {
            type Id = ::stecs::archetype::EntityKey<Self>;
            type Ref<#lifetime> = #ident_ref #ty_generics_lifetime;
            type RefMut<#lifetime> = #ident_ref_mut #ty_generics_lifetime;
            type WorldData = ::stecs::archetype::Archetype<#ident_columns #ty_generics>;
            type Fetch<#lifetime> = #ident_ref_fetch #ty_generics_lifetime;
            type FetchMut<#lifetime> = #ident_ref_mut_fetch #ty_generics_lifetime;
            type FetchId<'__stecs__w> = ::stecs::query::fetch::EntityKeyFetch<#ident #ty_generics>;
        }

        // EntityFromRef

        impl #impl_generics ::stecs::EntityFromRef for #ident #ty_generics
        where
            // https://github.com/rust-lang/rust/issues/48214#issuecomment-1150463333
            #(for<'__stecs__a> #field_comp_tys: ::std::clone::Clone,)*
            #(for<'__stecs__a> #field_flat_tys: ::stecs::EntityFromRef,)*

            #where_clause_predicates
        {
            fn from_ref(entity: Self::Ref<'_>) -> Self
            {
                Self {
                    #(#field_comp_idents: ::std::clone::Clone::clone(entity.#field_comp_idents),)*
                    #(#field_flat_idents:
                        <#field_flat_tys as ::stecs::EntityFromRef>::from_ref(
                            entity.#field_flat_idents,
                        ),
                    )*
                }
            }
        }

        // EntityVariant

        impl #impl_generics ::stecs::entity::EntityVariant<#ident #ty_generics>
        for #ident #ty_generics #where_clause {
            fn into_outer(self) -> Self {
                self
            }

            fn spawn(self, data: &mut Self::WorldData) -> ::stecs::EntityId<Self> {
                use ::stecs::WorldData;

                data.spawn(self)
            }

            fn id_to_outer(id: Self::Id) -> Self::Id {
                id
            }

            fn try_id_from_outer(id: Self::Id) -> ::std::option::Option<Self::Id> {
                Some(id)
            }
        }

        // EntityStruct

        impl #impl_generics ::stecs::entity::EntityStruct for #ident #ty_generics #where_clause {
            type Columns = #ident_columns #ty_generics;
        }

        // Columns

        #[allow(unused, non_camel_case_types)]
        #columns_derives
        #vis struct #ident_columns #impl_generics #where_clause {
            #(#field_comp_idents: ::stecs::column::Column<#field_comp_tys>,)*
            #(#field_flat_idents: ::stecs::entity::EntityColumns<#field_flat_tys>,)*
        }

        impl #impl_generics ::std::default::Default
        for #ident_columns #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#field_comp_idents: ::std::default::Default::default(),)*
                    #(#field_flat_idents: ::std::default::Default::default(),)*
                }
            }
        }

        impl #impl_generics ::stecs::entity::Columns
        for #ident_columns #ty_generics #where_clause {
            type Entity = #ident #ty_generics;

            fn column<__stecs__C: ::stecs::Component>(
                &self,
            ) -> ::std::option::Option<&::stecs::column::Column<__stecs__C>> {
                let mut result = ::std::option::Option::None;
                #(
                    result = result.or_else(||
                        ::stecs::column::downcast_ref(&self.#field_comp_idents)
                    );
                )*
                #(
                    result = result.or_else(||
                        self.#field_flat_idents.column::<__stecs__C>()
                    );
                )*

                result
            }

            fn push(&mut self, entity: Self::Entity) {
                #(self.#field_comp_idents.push(entity.#field_comp_idents);)*
                #(self.#field_flat_idents.push(entity.#field_flat_idents);)*
            }

            fn remove(&mut self, index: usize) -> Self::Entity {
                #ident {
                    #(#field_comp_idents: self.#field_comp_idents.remove(index),)*
                    #(#field_flat_idents: self.#field_flat_idents.remove(index),)*
                }
            }

            fn new_fetch<#lifetime>(
                &self,
                len: usize,
            ) -> <Self::Entity as ::stecs::entity::Entity>::Fetch<#lifetime> {
                #(::std::assert_eq!(len, self.#field_comp_idents.len());)*

                #ident_ref_fetch {
                    #(#field_comp_idents: self.#field_comp_idents.as_raw_parts(),)*
                    #(#field_flat_idents: self.#field_flat_idents.new_fetch(len),)*
                    __stecs__len: len,
                    __stecs__phantom: ::std::marker::PhantomData,
                }
            }

            fn new_fetch_mut<#lifetime>(
                &self,
                len: usize,
            ) -> <Self::Entity as ::stecs::entity::Entity>::FetchMut<#lifetime> {
                #(::std::assert_eq!(len, self.#field_comp_idents.len());)*

                #ident_ref_mut_fetch {
                    #(#field_comp_idents: self.#field_comp_idents.as_raw_parts_mut(),)*
                    #(#field_flat_idents: self.#field_flat_idents.new_fetch_mut(len),)*
                    __stecs__len: len,
                    __stecs__phantom: ::std::marker::PhantomData,
                }
            }

            fn push_flat_columns<#lifetime>(
                &#lifetime self,
                output: &mut ::std::vec::Vec<&#lifetime dyn ::std::any::Any>,
            ) {
                #(
                    output.push(&self.#field_flat_idents);
                    self.#field_flat_idents.push_flat_columns(output);
                )*
            }
        }

        // Ref

        #[allow(unused, non_snake_case, non_camel_case_types)]
        #ref_derives
        #vis struct #ident_ref #impl_generics_lifetime #where_clause_lifetime {
            #(#vis #field_comp_idents: &#lifetime #field_comp_tys,)*
            #(#vis #field_flat_idents: ::stecs::EntityRef<#lifetime, #field_flat_tys>,)*
            __stecs__phantom: ::std::marker::PhantomData<&#lifetime ()>,
        }

        impl #impl_generics_lifetime ::std::clone::Clone for #ident_ref #ty_generics_lifetime
        #where_clause_lifetime
        {
            fn clone(&self) -> Self {
                Self {
                    #(#field_comp_idents: self.#field_comp_idents,)*
                    #(#field_flat_idents: self.#field_flat_idents.clone(),)*
                    __stecs__phantom: ::std::marker::PhantomData,
                }
            }
        }

        // RefMut

        #[allow(unused, non_snake_case, non_camel_case_types)]
        #vis struct #ident_ref_mut #impl_generics_lifetime #where_clause_lifetime {
            #(#vis #field_comp_idents: &#lifetime mut #field_comp_tys,)*
            #(#vis #field_flat_idents: ::stecs::EntityRefMut<#lifetime, #field_flat_tys>,)*
            __stecs__phantom: ::std::marker::PhantomData<&#lifetime mut ()>,
        }

        // RefFetch

        #[allow(unused, non_snake_case, non_camel_case_types)]
        #vis struct #ident_ref_fetch #impl_generics_lifetime #where_clause_lifetime {
            #(#field_comp_idents: ::stecs::column::ColumnRawParts<#field_comp_tys>,)*
            #(#field_flat_idents: <#field_flat_tys as ::stecs::Entity>::Fetch<#lifetime>,)*
            __stecs__len: usize,
            __stecs__phantom: ::std::marker::PhantomData<&#lifetime ()>,
        }

        impl #impl_generics_lifetime ::std::clone::Clone
        for #ident_ref_fetch #ty_generics_lifetime #where_clause_lifetime {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl #impl_generics_lifetime ::std::marker::Copy
        for #ident_ref_fetch #ty_generics_lifetime #where_clause_lifetime {}

        unsafe impl #impl_generics_lifetime ::stecs::query::fetch::Fetch
        for #ident_ref_fetch #ty_generics_lifetime #where_clause_lifetime {
            type Item<#lifetime2> = #ident_ref #ty_generics_lifetime2 where Self: #lifetime2;

            fn new<__stecs__T: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &__stecs__T,
                strict_entity_ref: bool,
            ) -> ::std::option::Option<Self> {
                let mut flat_columns = ::std::vec::Vec::new();
                flat_columns.push(columns as &dyn ::std::any::Any);
                if !strict_entity_ref {
                    ::stecs::entity::Columns::push_flat_columns(columns, &mut flat_columns);
                }

                // FIXME: This assumes that no entity struct contains the same
                // entity type nested twice. Similar to duplicate components, we
                // need to check against this at `World` construction time.
                for flat_column in flat_columns {
                    let flat_column: ::std::option::Option<&#ident_columns #ty_generics> =
                        flat_column.downcast_ref();

                    let fetch = flat_column.map(|flat_column|
                        <#ident_columns #ty_generics as ::stecs::entity::Columns>
                        ::new_fetch(flat_column, ids.len())
                    );

                    if fetch.is_some() {
                        return fetch;
                    }
                }

                None
            }

            fn len(&self) -> usize {
                self.__stecs__len
            }

            unsafe fn get<#lifetime2>(&self, index: usize) -> Self::Item<#lifetime2>
            where
                Self: #lifetime2,
            {
                ::std::debug_assert!(index < self.len());

                #ident_ref {
                    #(#field_comp_idents: &*self.#field_comp_idents.ptr.add(index),)*
                    #(#field_flat_idents: self.#field_flat_idents.get(index),)*
                    __stecs__phantom: ::std::marker::PhantomData,
                }
            }
        }

        unsafe impl #impl_generics_lifetime ::stecs::Query
        for #ident_ref #ty_generics_lifetime #where_clause {
            type Fetch<#lifetime2> = #ident_ref_fetch #ty_generics_lifetime2;

            fn for_each_borrow(mut f: impl FnMut(::std::any::TypeId, bool)) {
                #(f(::std::any::TypeId::of::<#field_comp_tys>(), false);)*
                #(<#field_flat_tys as ::stecs::Entity>::Ref::<#lifetime>::for_each_borrow(&mut f);)*
            }
        }

        unsafe impl #impl_generics_lifetime ::stecs::QueryShared
        for #ident_ref #ty_generics_lifetime #where_clause {}

        // RefMutFetch

        #[allow(unused, non_snake_case, non_camel_case_types)]
        #vis struct #ident_ref_mut_fetch #impl_generics_lifetime #where_clause_lifetime {
            #(#field_comp_idents: ::stecs::column::ColumnRawPartsMut<#field_comp_tys>,)*
            #(#field_flat_idents: <#field_flat_tys as ::stecs::Entity>::FetchMut<#lifetime>,)*
            __stecs__len: usize,
            __stecs__phantom: ::std::marker::PhantomData<&#lifetime ()>,
        }

        impl #impl_generics_lifetime ::std::clone::Clone
        for #ident_ref_mut_fetch #ty_generics_lifetime #where_clause_lifetime {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl #impl_generics_lifetime ::std::marker::Copy
        for #ident_ref_mut_fetch #ty_generics_lifetime #where_clause_lifetime {}

        unsafe impl #impl_generics_lifetime ::stecs::query::fetch::Fetch
        for #ident_ref_mut_fetch #ty_generics_lifetime #where_clause_lifetime {
            type Item<#lifetime2> = #ident_ref_mut #ty_generics_lifetime2 where Self: #lifetime2;

            fn new<__stecs__T: ::stecs::entity::Columns>(
                ids: &::stecs::column::Column<::stecs::thunderdome::Index>,
                columns: &__stecs__T,
                strict_entity_ref: bool,
            ) -> ::std::option::Option<Self> {
                let mut flat_columns = ::std::vec::Vec::new();
                flat_columns.push(columns as &dyn ::std::any::Any);
                if !strict_entity_ref {
                    ::stecs::entity::Columns::push_flat_columns(columns, &mut flat_columns);
                }

                // FIXME: This assumes that no entity struct contains the same
                // entity type nested twice. Similar to duplicate components, we
                // need to check against this at `World` construction time.
                for flat_column in flat_columns {
                    let flat_column: ::std::option::Option<&#ident_columns #ty_generics> =
                        flat_column.downcast_ref();

                    let fetch = flat_column.map(|flat_column|
                        <#ident_columns #ty_generics as ::stecs::entity::Columns>
                        ::new_fetch_mut(flat_column, ids.len())
                    );

                    if fetch.is_some() {
                        return fetch;
                    }
                }

                None
            }

            fn len(&self) -> usize {
                self.__stecs__len
            }

            unsafe fn get<#lifetime2>(&self, index: usize) -> Self::Item<#lifetime2>
            where
                Self: #lifetime2,
            {
                ::std::debug_assert!(index < self.len());

                #ident_ref_mut {
                    #(#field_comp_idents: &mut *self.#field_comp_idents.ptr.add(index),)*
                    #(#field_flat_idents: self.#field_flat_idents.get(index),)*
                    __stecs__phantom: ::std::marker::PhantomData,
                }
            }
        }

        unsafe impl #impl_generics_lifetime ::stecs::Query
        for #ident_ref_mut #ty_generics_lifetime #where_clause {
            type Fetch<#lifetime2> = #ident_ref_mut_fetch #ty_generics_lifetime2;

            fn for_each_borrow(mut f: impl FnMut(::std::any::TypeId, bool)) {
                #(f(::std::any::TypeId::of::<#field_comp_tys>(), true);)*
                #(
                    <#field_flat_tys as ::stecs::Entity>::RefMut::<#lifetime>::for_each_borrow(
                        &mut f,
                    );
                )*
            }
      }
    })
}
