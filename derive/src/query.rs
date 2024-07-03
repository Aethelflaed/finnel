use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

mod param;
use param::Param;

mod query;
use query::{EntityRef, Query};

pub fn impl_query(input: DeriveInput) -> Result<TokenStream> {
    let query = Query::read(input)?;
    let Query {
        entity: EntityRef { entity, .. },
        result,
        ident: struct_ident,
        params,
        ..
    } = &query;

    let mut sql_query = quote! {
        let mut sql_query = String::from("SELECT\n");

    };

    {
        let mut join = "";
        for entity in &query.entities {
            let field_names = entity.field_names();
            let alias = &entity.alias;

            sql_query.extend(quote! {
                let mut join = #join;
                for field in #field_names {
                    sql_query.push_str(format!("{}\t{}.{} AS {}_{}",
                            join, #alias, field, #alias, field
                            ).as_str());
                    join = ",\n"
                }
            });

            join = ",\n"
        }
    }

    {
        let (table_name, alias) = query.entity.get();

        sql_query.extend(quote!{
            sql_query.push_str(format!("\nFROM {} AS {}\n", #table_name, #alias).as_str());
        });
    }

    for join in &query.joins {
        let clause = join.join_clause();
        sql_query.extend(quote! {
            sql_query.push_str(#clause.as_str());
        });
    }

    let mut parameters = quote! {
        let mut params = Vec::<(&str, &dyn ToSql)>:: new();
    };
    let mut validations = quote!();
    let mut join = "WHERE\n\t";

    let mut limit_param = Option::<&Param>::None;

    for param in params {
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
            sql_query.extend(param.add_query_criterion(join));
            parameters.extend(param.add_push_expr());
            validations.extend(param.as_validation());

            join = " AND\n\t";
        }
    }

    if let Some(param) = &limit_param {
        sql_query.extend(param.add_query_limit());
        parameters.extend(param.add_push_expr());
    }

    sql_query.extend(quote!(sql_query));
    parameters.extend(quote!(params));

    Ok(quote! {
        impl Query<#result, #entity> for #struct_ident {
            fn query(&self) -> String {
                #sql_query
            }

            fn params(&self) -> Vec<(&str, &dyn ToSql)> {
                #parameters
            }

            fn valid(&self) -> db::Result<()> {
                #validations

                Ok(())
            }
        }
    })
}

pub fn impl_query_debug(input: DeriveInput) -> Result<TokenStream> {
    let query = Query::read(input)?;
    let struct_name = query.name();

    let mut debug = quote! {
        f.debug_struct(#struct_name)
    };
    let mut params = quote! {
        let mut params = Vec::<(&str, String, String)>:: new();
    };

    for param in query.params {
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

    let struct_ident = query.ident;

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
