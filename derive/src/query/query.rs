use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, Error, Ident, LitStr, Result};

use super::Param;

#[derive(Clone)]
pub struct EntityRef {
    pub entity: Ident,
    pub alias: String,
}

impl EntityRef {
    pub fn new(entity: Ident, alias: Option<LitStr>) -> Self {
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

#[derive(Default)]
struct JoinAttr {
    pub clause: Option<LitStr>,
    pub lhs: Option<(EntityRef, String)>,
    pub rhs: Option<(EntityRef, String)>,
}

impl JoinAttr {
    fn parse(attr: &syn::Attribute) -> Result<Join> {
        let mut parsed = Self::default();

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("type") {
                parsed.clause = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("lhs") {
                parsed.lhs = Some(SideAttr::parse("lhs", &meta)?);
                return Ok(());
            }

            if meta.path.is_ident("rhs") {
                parsed.rhs = Some(SideAttr::parse("rhs", &meta)?);
                return Ok(());
            }

            Err(meta.error("unrecognized join attribute"))
        })?;

        let Some(clause) = parsed.clause else {
            return Err(Error::new(attr.meta.span(), "type not defined"));
        };
        let Some(lhs) = parsed.lhs else {
            return Err(Error::new(attr.meta.span(), "lhs not defined"));
        };
        let Some(rhs) = parsed.rhs else {
            return Err(Error::new(attr.meta.span(), "rhs not defined"));
        };

        Ok(Join {
            clause: clause.value(),
            lhs,
            rhs,
        })
    }
}

#[derive(Default)]
struct SideAttr {
    pub entity: Option<Ident>,
    pub alias: Option<LitStr>,
    pub field: Option<LitStr>,
}

impl SideAttr {
    fn parse(
        side: &str,
        meta: &syn::meta::ParseNestedMeta<'_>,
    ) -> Result<(EntityRef, String)> {
        let mut parsed = Self::default();

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("entity") {
                parsed.entity = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("alias") {
                parsed.alias = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("field") {
                parsed.field = Some(meta.value()?.parse()?);
                return Ok(());
            }

            Err(meta.error(format!("unrecognized {} attribute", side)))
        })?;

        let Some(field) = parsed.field else {
            return Err(meta.error("field not defined"));
        };

        let Some(entity) = parsed.entity else {
            return Err(meta.error("entity not defined"));
        };

        let entity = EntityRef::new(entity, parsed.alias);

        Ok((entity, field.value()))
    }
}

pub struct Join {
    pub clause: String,
    pub lhs: (EntityRef, String),
    pub rhs: (EntityRef, String),
}

impl Join {
    pub fn join_clause(&self) -> TokenStream {
        let clause = &self.clause;
        let lhs_alias = &self.lhs.0.alias;
        let lhs_field = &self.lhs.1;
        let rhs_alias = &self.rhs.0.alias;
        let rhs_field = &self.rhs.1;
        let rhs_table_name = self.rhs.0.table_name();

        quote! {
            format!("{} JOIN {} AS {}\n\tON {}.{} = {}.{}\n",
                #clause, #rhs_table_name, #rhs_alias,
                #lhs_alias, #lhs_field, #rhs_alias, #rhs_field)
        }
    }
}

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
            ident: ident,
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

        let mut query: Self = QueryAttr::parse(ident, &query_attr)?;

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
