use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, Error, Ident, LitStr, Result};

mod param;
use param::Param;

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

pub fn impl_query(input: DeriveInput) -> Result<TokenStream> {
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
        let param = Param::read(field)?;

        if param.limit() {
            if limit_param.is_some() {
                return Err(Error::new(
                    field.span(),
                    "Only one limit param allowed per query",
                ));
            }
            limit_param = Some(param);
        } else if param.ignore() {
            // no-op
        } else {
            query.extend(param.add_query_criterion(join));
            parameters.extend(param.add_push_expr());
            validations.extend(param.as_validation());

            join = " AND ";
        }
    }

    if let Some(param) = &limit_param {
        query.extend(param.add_query_limit());
        parameters.extend(param.add_push_expr());
    }

    query.extend(quote!(query));
    parameters.extend(quote!(params));

    let struct_ident = &input.ident;

    let expanded = quote! {
        impl Query<#entity> for #struct_ident {
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

pub fn impl_query_debug(input: DeriveInput) -> Result<TokenStream> {
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

    let struct_name = input.ident.to_string();

    let mut debug = quote! {
        f.debug_struct(#struct_name)
    };
    let mut params = quote! {
        let mut params = Vec::<(&str, String, String)>:: new();
    };

    for field in &data.named {
        let param = Param::read(field)?;

        let var = param.var_name();
        let ident = &param.ident();

        if !param.ignore() {
            params.extend(quote! {
                if let Some(value) = &self.#ident {
                    let sql = match value.to_sql().unwrap().to_sql().unwrap() {
                        rusqlite::types::ToSqlOutput::Borrowed(v) => match v {
                            rusqlite::types::ValueRef::Text(text) => {
                                format!("Text({})", std::str::from_utf8(text).unwrap())
                            },
                            v => format!("{:?}", v),
                        },
                        o => format!("{:?}", o),
                    };
                    params.push(
                        (
                            #var,
                            format!("{:?}", value),
                            sql,
                        )
                    );
                }
            });
        }
    }

    debug.extend(quote! {
        .field("query", &self.query())
        .field("params", &self.params_debug())
    });

    let struct_ident = &input.ident;

    Ok(quote! {
        impl #struct_ident {
            fn params_debug(&self) -> Vec<(&str, String, String)> {
                #params
                params
            }
        }

        impl std::fmt::Debug for #struct_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #debug.finish()
            }
        }
    })
}
