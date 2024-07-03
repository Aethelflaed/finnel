use syn::spanned::Spanned;
use syn::{DeriveInput, Error, FieldsNamed, Ident, LitStr, Result};

use super::Field;

pub struct Entity {
    pub ident: Ident,
    pub table: LitStr,
    data: FieldsNamed,
}

impl Entity {
    pub fn read(input: &DeriveInput) -> Result<Entity> {
        let syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(data),
            ..
        }) = &input.data
        else {
            return Err(Error::new(
                input.span(),
                "Entity derive is only available on struct with named fields",
            ));
        };

        let ident = input.ident.clone();

        let attr = match input
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("entity"))
        {
            Some(attr) => attr,
            _ => {
                return Err(Error::new(
                    input.span(),
                    "Missing attribute entity",
                ));
            }
        };

        let mut table: Option<LitStr> = None;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                table = Some(meta.value()?.parse()?);
                return Ok(());
            }

            Err(meta.error("unrecognized query attribute"))
        })?;

        let Some(table) = table else {
            return Err(Error::new(attr.meta.span(), "table not defined"));
        };

        Ok(Entity {
            ident,
            table,
            data: data.clone(),
        })
    }

    pub fn name(&self) -> String {
        self.ident.to_string()
    }

    pub fn fields(&self) -> Iter<'_> {
        Iter {
            iter: self.data.named.iter(),
        }
    }
}

pub struct Iter<'a> {
    iter: syn::punctuated::Iter<'a, syn::Field>,
}

impl Iterator for Iter<'_> {
    type Item = Result<Field>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Field::read)
    }
}
