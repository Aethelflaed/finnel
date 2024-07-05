use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Ident, LitStr, Result};

use super::EntityRef;

#[derive(Default)]
pub struct JoinAttr {
    pub clause: Option<LitStr>,
    pub lhs: Option<(EntityRef, String)>,
    pub rhs: Option<(EntityRef, String)>,
}

impl JoinAttr {
    pub fn parse(attr: &syn::Attribute) -> Result<Join> {
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

/// #[entity(entity = Record, select(all))]
/// #[entity(entity = Record, select(all, add = ["field", ("COUNT(x)", "alias")]))]
#[derive(Default)]
struct SelectAttr {
    pub all: bool,
    pub none: bool,
    pub add: Option<syn::ExprArray>,
    pub remove: Option<syn::ExprArray>,
}

impl SelectAttr {
    fn parse(
        meta: &syn::meta::ParseNestedMeta<'_>,
    ) -> Result<Self> {
        let mut parsed = Self::default();

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("all") {
                return parsed.set_all().map_err(|e| meta.error(e));
            }
            if meta.path.is_ident("none") {
                return parsed.set_none().map_err(|e| meta.error(e));
            }
            if meta.path.is_ident("add") {
                parsed.add = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("remove") {
                parsed.remove = Some(meta.value()?.parse()?);
                return Ok(());
            }

            Err(meta.error("unrecognized select attribute"))
        })?;

        Ok(parsed)
    }

    fn set_all(&mut self) -> std::result::Result<(), &str> {
        if self.none {
            return Err("cannot be both all and none");
        }
        self.all = true;
        Ok(())
    }

    fn set_none(&mut self) -> std::result::Result<(), &str> {
        if self.all {
            return Err("cannot be both none and all");
        }
        self.none = true;
        Ok(())
    }
}

/// #[join(rhs(entity = Entity, alias = "foo", field = "id", select(..)))]
#[derive(Default)]
struct SideAttr {
    pub entity: Option<Ident>,
    pub alias: Option<LitStr>,
    pub field: Option<LitStr>,
    pub select: Option<SelectAttr>,
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

            if meta.path.is_ident("select") {
                parsed.select = Some(SelectAttr::parse(&meta)?);
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

