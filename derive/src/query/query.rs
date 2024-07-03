use syn::spanned::Spanned;
use syn::{DeriveInput, Error, Field, FieldsNamed, Ident, LitStr, Result};

use super::Param;

pub struct EntityRef {
    pub entity: Ident,
    pub alias: String,
}

pub struct Query {
    pub ident: Ident,
    pub result: Ident,
    pub entity: EntityRef,
    data: FieldsNamed,
}

impl Query {
    pub fn read(input: DeriveInput) -> Result<Query> {
        let span = input.span();

        let DeriveInput {
            data:
                syn::Data::Struct(syn::DataStruct {
                    fields: syn::Fields::Named(data),
                    ..
                }),
            ident,
            attrs,
            ..
        } = input
        else {
            return Err(Error::new(
                span,
                "Query derive is only available on struct with named fields",
            ));
        };

        let attr = match attrs.iter().find(|attr| attr.path().is_ident("query"))
        {
            Some(attr) => attr,
            _ => {
                return Err(Error::new(span, "Missing attribute query"));
            }
        };

        let mut entity: Option<Ident> = None;
        let mut result: Option<Ident> = None;
        let mut alias: Option<LitStr> = None;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("entity") {
                entity = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("result") {
                result = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("alias") {
                alias = Some(meta.value()?.parse()?);
                return Ok(());
            }

            Err(meta.error("unrecognized query attribute"))
        })?;

        let Some(entity) = entity else {
            return Err(Error::new(attr.meta.span(), "entity not defined"));
        };
        let result = result.unwrap_or(entity.clone());
        let alias = alias.map(|a| a.value()).unwrap_or(entity.to_string());

        Ok(Query {
            ident,
            entity: EntityRef { entity, alias },
            result,
            data,
        })
    }

    pub fn name(&self) -> String {
        self.ident.to_string()
    }

    pub fn params(&self) -> Iter<'_> {
        Iter {
            entity: &self.entity,
            iter: self.data.named.iter(),
        }
    }
}

pub struct Iter<'a> {
    entity: &'a EntityRef,
    iter: syn::punctuated::Iter<'a, Field>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Result<Param<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|field| Param::read(self.entity, field))
    }
}
