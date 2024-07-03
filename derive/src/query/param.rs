use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Error, Expr, Field, Ident, Result};

pub struct Param {
    syn_field: Field,
    ident: Ident,
    mandatory: bool,
    ignore: bool,
    limit: bool,
    operator: Option<Expr>,
    field: Option<Expr>,
}

impl Param {
    pub fn read(input: &Field) -> Result<Param> {
        let mut param = Param {
            syn_field: input.clone(),
            ident: input.ident.clone().unwrap(),
            mandatory: false,
            ignore: false,
            limit: false,
            operator: None,
            field: None,
        };

        if let Some(attr) = input
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("param"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("mandatory") {
                    return param.set_mandatory().map_err(|e| meta.error(e));
                }

                if meta.path.is_ident("ignore") {
                    return param.set_ignore().map_err(|e| meta.error(e));
                }

                if meta.path.is_ident("limit") {
                    return param.set_limit().map_err(|e| meta.error(e));
                }

                if meta.path.is_ident("operator") {
                    param.operator = Some(meta.value()?.parse()?);
                    return Ok(());
                }

                if meta.path.is_ident("field") {
                    param.field = Some(meta.value()?.parse()?);
                    return Ok(());
                }

                Err(meta.error("unrecognized query attribute"))
            })?;
        }

        Ok(param)
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

    pub fn operator(&self) -> TokenStream {
        if let Some(op) = &self.operator {
            op.to_token_stream()
        } else {
            quote!("=")
        }
    }

    pub fn field(&self) -> TokenStream {
        if let Some(f) = &self.field {
            f.to_token_stream()
        } else {
            let name = self.name();
            quote!(#name)
        }
    }

    pub fn var_name(&self) -> TokenStream {
        let name = format!(":{}", self.name());

        quote! {#name}
    }

    pub fn as_condition(&self) -> TokenStream {
        let ident = &self.ident;
        let field = self.field();
        let operator = self.operator();
        let var = self.var_name();

        quote! {
            if let Some(_) = &self.#ident {
                format!("{} {} {}", #field, #operator, #var)
            } else { String::new() }
        }
    }

    pub fn add_push_expr(&self) -> TokenStream {
        let ident = &self.ident;
        let var = self.var_name();

        quote! {
            if let Some(value) = &self.#ident {
                params.push((#var, value as &dyn ToSql));
            }
        }
    }

    pub fn as_validation(&self) -> TokenStream {
        if !self.mandatory {
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
        if !self.limit {
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
