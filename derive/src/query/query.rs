use syn::spanned::Spanned;
use syn::{DeriveInput, Error, Ident, LitStr, Result};

use super::{Param, EntityRef, Join, JoinAttr};

pub struct Query {
    pub ident: Ident,
    pub result: Ident,
    pub entity: EntityRef,
    pub params: Vec<Param>,
    pub joins: Vec<Join>,
    pub entities: Vec<EntityRef>,
}

#[derive(Default)]
pub struct QueryAttr {
    alias: Option<LitStr>,
    entity: Option<Ident>,
    result: Option<Ident>,
}

impl QueryAttr {
    fn parse(ident: Ident, attr: &syn::Attribute) -> Result<Query> {
        let mut parsed = Self::default();

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("entity") {
                parsed.entity = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("result") {
                parsed.result = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("alias") {
                parsed.alias = Some(meta.value()?.parse()?);
                return Ok(());
            }

            Err(meta.error("unrecognized query attribute"))
        })?;

        let Some(entity) = parsed.entity else {
            return Err(Error::new(attr.meta.span(), "entity not defined"));
        };

        let result = parsed.result.unwrap_or(entity.clone());
        let entity = EntityRef::new(entity, parsed.alias);

        Ok(Query {
            ident,
            entity,
            result,
            params: Vec::new(),
            joins: Vec::new(),
            entities: Vec::new(),
        })
    }
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

        let query_attr =
            match attrs.iter().find(|attr| attr.path().is_ident("query")) {
                Some(attr) => attr,
                _ => {
                    return Err(Error::new(span, "Missing attribute query"));
                }
            };

        let mut query: Self = QueryAttr::parse(ident, query_attr)?;

        query.entities.push(query.entity.clone());

        for attr in attrs {
            if attr.path().is_ident("join") {
                let join = JoinAttr::parse(&attr)?;
                query.entities.push(join.rhs.0.clone());
                query.joins.push(join);
            }
        }

        query.params = data
            .named
            .iter()
            .map(|field| Param::read(query.entity.clone(), field))
            .collect::<Result<Vec<_>>>()?;

        Ok(query)
    }

    pub fn name(&self) -> String {
        self.ident.to_string()
    }
}
