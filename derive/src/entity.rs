use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

mod entity;
use entity::Entity;

mod field;
use field::Field;

pub fn impl_entity_descriptor(input: DeriveInput) -> Result<TokenStream> {
    let entity = Entity::read(&input)?;
    let Entity {
        ident: struct_ident,
        table,
        ..
    } = &entity;

    let mut field_names = quote!();
    for result in entity.fields() {
        let field = result?;
        let name = field.name();

        field_names.extend(quote!(#name,));
    }

    Ok(quote! {
        impl db::entity::EntityDescriptor for #struct_ident {
            fn table_name() -> &'static str {
                #table
            }

            fn field_names() -> &'static [&'static str] {
                &[
                    #field_names
                ]
            }
        }
    })
}
