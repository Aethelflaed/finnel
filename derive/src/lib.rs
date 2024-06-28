use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Error, Expr, Ident, LitStr, Result};

#[proc_macro_derive(Query, attributes(query, param))]
pub fn derive_query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    impl_query(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn read_query_attribute(input: &DeriveInput) -> Result<(Ident, LitStr)> {
    let attr = match input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("query"))
    {
        Some(attr) => attr,
        _ => {
            return Err(Error::new(input.span(), "Missing attribute query"));
        }
    };

    let mut entity: Option<Ident> = None;
    let mut table: Option<LitStr> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("entity") {
            entity = Some(meta.value()?.parse()?);
            return Ok(());
        }

        if meta.path.is_ident("table") {
            table = Some(meta.value()?.parse()?);
            return Ok(());
        }

        Err(meta.error("unrecognized query attribute"))
    })?;

    let Some(entity) = entity else {
        return Err(Error::new(attr.meta.span(), "entity not defined"));
    };
    let Some(table) = table else {
        return Err(Error::new(attr.meta.span(), "table not defined"));
    };

    Ok((entity, table))
}

fn read_param(input: &syn::Field) -> Result<Param> {
    let mut param = Param::new(input.ident.clone().unwrap());

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

#[derive(Debug)]
struct Param {
    name: Ident,
    mandatory: bool,
    ignore: bool,
    limit: bool,
    operator: Option<Expr>,
    field: Option<Expr>,
}

impl Param {
    fn new(name: Ident) -> Self {
        Self {
            name,
            mandatory: false,
            ignore: false,
            limit: false,
            operator: None,
            field: None,
        }
    }

    fn set_mandatory(&mut self) -> std::result::Result<(), &str> {
        if self.ignore {
            return Err("cannot be both mandatory and ignore");
        } else if self.limit {
            return Err("cannot be both mandatory and limit");
        }
        self.mandatory = true;
        Ok(())
    }

    fn set_ignore(&mut self) -> std::result::Result<(), &str> {
        if self.mandatory {
            return Err("cannot be both ignore and mandatory");
        } else if self.limit {
            return Err("cannot be both ignore and limit");
        }
        self.ignore = true;
        Ok(())
    }

    fn set_limit(&mut self) -> std::result::Result<(), &str> {
        if self.ignore {
            return Err("cannot be both limit and ignore");
        } else if self.mandatory {
            return Err("cannot be both limit and mandatory");
        }
        self.limit = true;
        Ok(())
    }

    fn operator(&self) -> TokenStream {
        if let Some(op) = &self.operator {
            op.to_token_stream()
        } else {
            quote!("=")
        }
    }

    fn field(&self) -> TokenStream {
        if let Some(f) = &self.field {
            f.to_token_stream()
        } else {
            let name = &self.name.to_string();
            quote!(#name)
        }
    }

    fn var_name(&self) -> TokenStream {
        let name = format!(":{}", self.name);

        quote! {#name}
    }

    fn as_condition(&self) -> TokenStream {
        let name = &self.name;
        let field = self.field();
        let operator = self.operator();
        let var = self.var_name();

        quote! {
            if let Some(_) = &self.#name {
                format!("{} {} {}", #field, #operator, #var)
            } else { String::new() }
        }
    }

    fn as_push_expr(&self) -> TokenStream {
        let name = &self.name;
        let var = self.var_name();

        quote! {
            if let Some(value) = &self.#name {
                params.push((#var, value as &dyn ToSql));
            }
        }
    }

    fn as_validation(&self) -> TokenStream {
        if !self.mandatory {
            return quote!();
        }

        let param = &self.name;
        let name = self.name.to_string();

        quote! {
            if self.#param.is_none() {
                return Err(db::Error::Invalid(format!("{} is mandatory", #name)));
            }
        }
    }
}

fn impl_query(input: DeriveInput) -> Result<TokenStream> {
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

    let (entity, table) = read_query_attribute(&input)?;

    let mut query = quote! {
        let mut query = format!("SELECT * FROM {}", #table);
    };
    let mut parameters = quote! {
        let mut params = Vec::<(&str, &dyn ToSql)>:: new();
    };
    let mut validations = quote!();
    let mut join = " WHERE ";

    let mut limit_param = Option::<Param>::None;

    for field in &data.named {
        let param = read_param(field)?;

        if param.limit {
            if limit_param.is_some() {
                return Err(Error::new(
                    field.span(),
                    "Only one limit param allowed per query",
                ));
            }
            limit_param = Some(param);
        } else if param.ignore {
            // no-op
        } else {
            let cond = param.as_condition();

            query.extend(quote! {
                let cond = #cond;
                if !cond.is_empty() {
                    query.push_str(format!("{}{}", #join, cond).as_str());
                }
            });

            join = " AND ";

            parameters.extend(param.as_push_expr());
            validations.extend(param.as_validation());
        }
    }

    if let Some(param) = &limit_param {
        let name = &param.name;
        let var = param.var_name();

        query.extend(quote! {
            if let Some(value) = self.#name  {
                query.push_str(format!(" LIMIT {}", #var).as_str());
            }
        });
        parameters.extend(param.as_push_expr());
    }

    query.extend(quote!(query));
    parameters.extend(quote!(params));

    let struct_name = &input.ident;

    let expanded = quote! {
        impl Query<#entity> for #struct_name {
            fn query(&self) -> String {
                #query
            }

            fn params(&self) -> Vec<(&str, &dyn ToSql)> {
                #parameters
            }

            fn valid(&self) -> db::Result<()> {
                #validations

                Ok(())
            }
        }
    };

    Ok(expanded)
}
