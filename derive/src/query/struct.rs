use syn::spanned::Spanned;
use syn::{DeriveInput, Error, Field, FieldsNamed, Ident, Result};

use super::Param;

pub struct Struct {
    pub ident: Ident,
    pub entity: Ident,
    pub result: Ident,
    data: FieldsNamed,
}

impl Struct {
    pub fn read(input: &DeriveInput) -> Result<Struct> {
        let syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(data),
            ..
        }) = &input.data
        else {
            return Err(Error::new(
                input.span(),
                "Query derive is only available on struct with named fields",
            ));
        };

        let ident = input.ident.clone();

        let attr = match input
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("query"))
        {
            Some(attr) => attr,
            _ => {
                return Err(Error::new(
                    input.span(),
                    "Missing attribute query",
                ));
            }
        };

        let mut entity: Option<Ident> = None;
        let mut result: Option<Ident> = None;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("entity") {
                entity = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("result") {
                result = Some(meta.value()?.parse()?);
                return Ok(());
            }

            Err(meta.error("unrecognized query attribute"))
        })?;

        let Some(entity) = entity else {
            return Err(Error::new(attr.meta.span(), "entity not defined"));
        };
        let result = result.unwrap_or(entity.clone());

        Ok(Struct {
            ident,
            entity,
            result,
            data: data.clone(),
        })
    }

    pub fn name(&self) -> String {
        self.ident.to_string()
    }

    pub fn params(&self) -> Iter<'_> {
        Iter {
            iter: self.data.named.iter(),
        }
    }
}

pub struct Iter<'a> {
    iter: syn::punctuated::Iter<'a, Field>,
}

impl Iterator for Iter<'_> {
    type Item = Result<Param>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Param::read)
    }
}
