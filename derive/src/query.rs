use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

mod param;
use param::Param;

mod r#struct;
use r#struct::Struct;

pub fn impl_query(input: DeriveInput) -> Result<TokenStream> {
    let query_struct = Struct::read(&input)?;
    let Struct {
        entity,
        result,
        ident: struct_ident,
        ..
    } = &query_struct;

    let mut query = quote! {
        let mut query = String::from("SELECT\n");
        let table_name = <#entity as db::entity::EntityDescriptor>::table_name();
        let fields = <#entity as db::entity::EntityDescriptor>::field_names().iter().map(|field| {
            format!("\t{table_name}.{field} AS {table_name}_{field}")
        }).collect::<Vec<String>>().join(",\n");
        query.push_str(format!("{fields}\nFROM {table_name}\n").as_str());
    };
    let mut parameters = quote! {
        let mut params = Vec::<(&str, &dyn ToSql)>:: new();
    };
    let mut validations = quote!();
    let mut join = "WHERE\n\t";

    let mut limit_param = Option::<Param>::None;

    for result in query_struct.params() {
        let param = result?;

        if param.limit() {
            if limit_param.is_some() {
                return Err(
                    param.error("Only one limit param allowed per query")
                );
            }
            limit_param = Some(param);
        } else if param.ignore() {
            // no-op
        } else {
            query.extend(param.add_query_criterion(join));
            parameters.extend(param.add_push_expr());
            validations.extend(param.as_validation());

            join = " AND\n\t";
        }
    }

    if let Some(param) = &limit_param {
        query.extend(param.add_query_limit());
        parameters.extend(param.add_push_expr());
    }

    query.extend(quote!(query));
    parameters.extend(quote!(params));

    let expanded = quote! {
        impl Query<#result, #entity> for #struct_ident {
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
    let query_struct = Struct::read(&input)?;
    let struct_name = query_struct.name();

    let mut debug = quote! {
        f.debug_struct(#struct_name)
    };
    let mut params = quote! {
        let mut params = Vec::<(&str, String, String)>:: new();
    };

    for result in query_struct.params() {
        let param = result?;

        let var = param.var_name();
        let ident = &param.ident();

        if !param.ignore() {
            params.extend(quote! {
                if let Some(value) = &self.#ident {
                    let sql = match value.to_sql().unwrap() {
                        rusqlite::types::ToSqlOutput::Borrowed(v) => match v {
                            rusqlite::types::ValueRef::Text(text) => {
                                format!("Text(\"{}\")", std::str::from_utf8(text).unwrap())
                            },
                            v => format!("{:?}", v),
                        },
                        rusqlite::types::ToSqlOutput::Owned(v) => format!("{:?}", v),
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
                f.write_str(self.query().as_str())?;
                f.write_str("\n")?;
                #debug.finish()
            }
        }
    })
}
