use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

mod entity;
use entity::Entity;

mod field;
use field::Field;

pub fn impl_entity(input: DeriveInput) -> Result<TokenStream> {
    let entity = Entity::read(&input)?;
    let Entity {
        ident: struct_ident,
        table,
        ..
    } = &entity;
    let table = table.value();

    let find_query = format!("SELECT * FROM {table} WHERE id = ? LIMIT 1;");

    let mut insert_query_start = format!("INSERT INTO {table} (\n");
    let mut insert_query_end = String::from("\n) VALUES (\n");
    let mut insert_join = "";

    let mut update_query = format!("UPDATE {table}\nSET\n");
    let mut update_join = "";

    let mut insert_params = quote!();
    let mut update_params = quote!();

    let mut fields_from_row = quote!();

    for result in entity.fields() {
        let field = result?;

        fields_from_row.extend(field.as_from_row());

        if field.insert() {
            insert_query_start
                .push_str(format!("{insert_join}\t{}", field.name()).as_str());
            insert_query_end.push_str(
                format!("{insert_join}\t{}", field.var_name()).as_str(),
            );
            insert_join = ",\n";

            insert_params.extend(field.as_param());
        }
        if field.update() {
            update_query.push_str(
                format!(
                    "{update_join}\t{} = {}",
                    field.name(),
                    field.var_name()
                )
                .as_str(),
            );
            update_join = ",\n";

            update_params.extend(field.as_param());
        }
    }

    let insert_query = format!(
        "{}{}\n)\nRETURNING id;",
        insert_query_start, insert_query_end
    );
    update_query.push_str("\nWHERE\n\tid = :id");

    Ok(quote! {
        impl Entity for #struct_ident {
            fn id(&self) -> Option<Id> {
                self.id
            }

            fn find(db: &Connection, id: Id) -> Result<Self> {
                let mut statement = db.prepare(#find_query)?;
                match statement.query_row([id], |row| Self::try_from(&Row::from(row))) {
                    Ok(record) => Ok(record),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
                    Err(e) => Err(e.into()),
                }
            }

            fn save(&mut self, db: &Connection) -> Result<()> {
                if let Some(id) = self.id() {
                    let mut statement = db.prepare(#update_query)?;
                    let params = rusqlite::named_params! {
                        ":id": id,
                        #update_params
                    };

                    match statement.execute(params) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.into()),
                    }
                } else {
                    let mut statement = db.prepare(#insert_query)?;
                    let params = rusqlite::named_params! {
                        #insert_params
                    };

                    Ok(statement.query_row(params, |row| {
                        self.id = row.get(0)?;
                        Ok(())
                    })?)
                }
            }
        }

        impl TryFrom<&Row<'_>> for #struct_ident {
            type Error = rusqlite::Error;

            fn try_from(row: &Row) -> rusqlite::Result<Self> {
                Ok(#struct_ident {
                    #fields_from_row
                })
            }
        }
    })
}

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
