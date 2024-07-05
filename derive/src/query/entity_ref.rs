use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

#[derive(Clone)]
pub struct EntityRef {
    pub entity: Ident,
    pub alias: String,
    // TODO: add table_name
    // TODO: add SelectAttr
    // if table_name is provided and we don't need to fetch the field names
    // (i.e. if select(none), although you can still select(none, add = [..])
    // then we don't need Entity to be a EntityDescriptor
}

impl EntityRef {
    pub fn new(entity: Ident, alias: Option<LitStr>) -> Self {
        // TODO make `alias` a TokenStream with `table_name()` as a default value
        let alias = alias.map(|a| a.value()).unwrap_or(entity.to_string());

        EntityRef { entity, alias }
    }

    pub fn get(&self) -> (TokenStream, &str) {
        (self.table_name(), self.alias.as_str())
    }

    pub fn table_name(&self) -> TokenStream {
        let entity = &self.entity;
        quote! {
            <#entity as db::entity::EntityDescriptor>::table_name()
        }
    }

    pub fn field_names(&self) -> TokenStream {
        let entity = &self.entity;
        quote! {
            <#entity as db::entity::EntityDescriptor>::field_names()
        }
    }
}
