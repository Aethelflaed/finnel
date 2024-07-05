use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Error, Expr, Field, Ident, LitStr, Result};

use super::EntityRef;

pub struct FieldRef {
    entity: EntityRef,
    field: TokenStream,
}

impl FieldRef {
    fn new(entity: EntityRef, ident: &Ident) -> FieldRef {
        let name = ident.to_string();

        FieldRef {
            entity,
            field: quote!(#name),
        }
    }

    pub fn get(&self) -> (&str, &TokenStream) {
        (self.entity.alias.as_str(), &self.field)
    }
}

#[derive(Default)]
pub struct ParamAttr {
    mandatory: bool,
    ignore: bool,
    limit: bool,
}

impl ParamAttr {
    pub fn mandatory(&self) -> bool {
        self.mandatory
    }

    pub fn set_mandatory(&mut self) -> std::result::Result<(), &str> {
        if self.ignore {
            return Err("cannot be both mandatory and ignore");
        } else if self.limit {
            return Err("cannot be both mandatory and limit");
        }
        self.mandatory = true;
        Ok(())
    }

    pub fn ignore(&self) -> bool {
        self.ignore
    }

    pub fn set_ignore(&mut self) -> std::result::Result<(), &str> {
        if self.mandatory {
            return Err("cannot be both ignore and mandatory");
        } else if self.limit {
            return Err("cannot be both ignore and limit");
        }
        self.ignore = true;
        Ok(())
    }

    pub fn limit(&self) -> bool {
        self.limit
    }

    pub fn set_limit(&mut self) -> std::result::Result<(), &str> {
        if self.ignore {
            return Err("cannot be both limit and ignore");
        } else if self.mandatory {
            return Err("cannot be both limit and mandatory");
        }
        self.limit = true;
        Ok(())
    }
}

pub struct Param {
    ident: Ident,
    field: FieldRef,
    span: Span,
    attr: ParamAttr,
    operator: String,
}

impl Param {
    pub fn read(entity: EntityRef, input: &Field) -> Result<Param> {
        let span = input.span();
        let mut param_attr = ParamAttr::default();
        let mut operator = Option::<LitStr>::None;

        let ident = input.ident.clone().unwrap();
        let mut field = FieldRef::new(entity, &ident);

        if let Some(attr) = input
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("param"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("mandatory") {
                    return param_attr
                        .set_mandatory()
                        .map_err(|e| meta.error(e));
                }

                if meta.path.is_ident("ignore") {
                    return param_attr.set_ignore().map_err(|e| meta.error(e));
                }

                // TODO: add offset
                if meta.path.is_ident("limit") {
                    return param_attr.set_limit().map_err(|e| meta.error(e));
                }

                if meta.path.is_ident("operator") {
                    operator = Some(meta.value()?.parse()?);
                    return Ok(());
                }

                if meta.path.is_ident("field") {
                    field.field =
                        meta.value()?.parse::<Expr>()?.into_token_stream();
                    return Ok(());
                }

                Err(meta.error("unrecognized param attribute"))
            })?;
        }

        let operator = operator.map(|op| op.value()).unwrap_or("=".to_string());

        Ok(Param {
            ident,
            span,
            attr: param_attr,
            operator,
            field,
        })
    }

    pub fn error(&self, message: &str) -> Error {
        Error::new(self.span, message)
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn name(&self) -> String {
        self.ident.to_string()
    }

    pub fn mandatory(&self) -> bool {
        self.attr.mandatory()
    }

    pub fn ignore(&self) -> bool {
        self.attr.ignore()
    }

    pub fn limit(&self) -> bool {
        self.attr.limit()
    }

    pub fn var_name(&self) -> TokenStream {
        let name = format!(":{}", self.name());

        quote! {#name}
    }

    pub fn as_condition(&self) -> TokenStream {
        let ident = &self.ident;
        let operator = &self.operator;
        let var = self.var_name();
        let (alias, field) = self.field.get();

        quote! {
            if let Some(_) = &self.#ident {
                format!("{}.{} {} {}", #alias, #field, #operator, #var)
            } else { String::new() }
        }
    }

    pub fn add_push_expr(&self) -> TokenStream {
        let ident = &self.ident;
        let var = self.var_name();

        quote! {
            if let Some(value) = &self.#ident {
                params.push((#var, value as &dyn rusqlite::ToSql));
            }
        }
    }

    pub fn as_validation(&self) -> TokenStream {
        if !self.mandatory() {
            return quote!();
        }

        let ident = &self.ident;
        let name = self.name();

        quote! {
            if self.#ident.is_none() {
                return Err(db::Error::Invalid(format!("{} is mandatory", #name)));
            }
        }
    }

    /// If the resolved condition for this parameter is not empty, add it
    /// to the query variable, separated with the given join
    pub fn add_query_criterion(&self, join: &str) -> TokenStream {
        let cond = self.as_condition();

        quote! {
            let cond = #cond;
            if !cond.is_empty() {
                sql_query.push_str(format!("{}{}", #join, cond).as_str());
            }
        }
    }

    pub fn add_query_limit(&self) -> TokenStream {
        if !self.limit() {
            return quote!();
        }

        let ident = &self.ident;
        let var = self.var_name();

        quote! {
            if let Some(value) = self.#ident  {
                sql_query.push_str(format!("\nLIMIT {}", #var).as_str());
            }
        }
    }
}
