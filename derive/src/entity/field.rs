use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Ident, LitBool, Path, Result};

pub struct Field {
    syn_field: syn::Field,
    ident: Ident,
    db_type: Option<Path>,
    insert: bool,
    update: bool,
}

impl Field {
    pub fn read(input: &syn::Field) -> Result<Field> {
        let mut field = Field {
            syn_field: input.clone(),
            ident: input.ident.clone().unwrap(),
            db_type: None,
            insert: true,
            update: true,
        };

        if let Some(attr) = input
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("field"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("db_type") {
                    field.db_type = Some(meta.value()?.parse()?);
                    return Ok(());
                }

                if meta.path.is_ident("insert") {
                    field.insert = meta.value()?.parse::<LitBool>()?.value();
                    return Ok(());
                }

                if meta.path.is_ident("update") {
                    field.update = meta.value()?.parse::<LitBool>()?.value();
                    return Ok(());
                }

                Err(meta.error("unrecognized field attribute"))
            })?;
        }

        if field.name() == "id" {
            field.insert = false;
            field.update = false;
        }

        Ok(field)
    }

    pub fn error(&self, message: &str) -> Error {
        Error::new(self.syn_field.span(), message)
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn name(&self) -> String {
        self.ident.to_string()
    }

    pub fn var_name(&self) -> String {
        format!(":{}", self.name())
    }

    pub fn db_type(&self) -> &Option<Path> {
        &self.db_type
    }

    pub fn insert(&self) -> bool {
        self.insert
    }

    pub fn update(&self) -> bool {
        self.update
    }

    pub fn as_param(&self) -> TokenStream {
        let ident = self.ident();
        let var_name = self.var_name();

        if let Some(db_type) = self.db_type() {
            quote! {
                #var_name: #db_type::from(self.#ident),
            }
        } else {
            quote! {
                #var_name: self.#ident,
            }
        }
    }

    pub fn as_from_row(&self) -> TokenStream {
        let ident = self.ident();
        let name = self.name();

        if let Some(db_type) = self.db_type() {
            quote! {
                #ident: row.get::<#db_type>(#name)?.into(),
            }
        } else {
            quote! {
                #ident: row.get(#name)?,
            }
        }
    }
}
