use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

mod entity;
mod query;

#[proc_macro_derive(Query, attributes(query, param))]
pub fn derive_query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    query::impl_query(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(QueryDebug, attributes(query, param))]
pub fn derive_query_debug(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    query::impl_query_debug(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(EntityDescriptor, attributes(entity, field))]
pub fn derive_entity_descriptor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    entity::impl_entity_descriptor(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
