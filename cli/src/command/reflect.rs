use crate::util::create_runtime;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use heck::{ToPascalCase, ToSnakeCase};
use itertools::Itertools;
use ormlite::Acquire;
use ormlite_core::config::get_var_database_url;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rust_format::{Formatter, PrettyPlease};
use sqlx::{FromRow, PgConnection, Connection};
use std::fs;

/* TODO
 * - Dynamically import required types
 * - Reserved words `r#` quoting
 * - Repeated field name detection
 * - Integrate `KVec` one-to-many implementation
 * - Properly support views
 * - Provide a mechanism to skip unwanted fields and other settings (via doc-comments or custom directive)
 * - `--split` option for generating one model per file
 * - `--domains` option for domains as `type` aliases
 */

#[allow(dead_code)]
#[derive(Debug, FromRow)]
struct ColumnDef {
    table_schema: String,
    table_name: String,
    column_name: String,
    is_rev_fk: bool,
    is_nullable: bool,
    type_name: String,
    is_array: bool,
    domain_schema: Option<String>,
    domain_name: Option<String>,
    is_primary_key: bool,
    fk_table_schema: Option<String>,
    fk_table_name: Option<String>,
    fk_column_name: Option<String>,
    is_updatable: bool,
}

impl ColumnDef {
    pub fn rust_type(&self) -> TokenStream {
        let base_type = match self.type_name.as_str() {
            "bool"            /* B:   16, 1000 */
                => quote! { bool },
            "date"            /* D: 1082, 1182 */
                => quote! { Date },
            "time"            /* D: 1083, 1183 */
            | "timetz"        /* D: 1266, 1270 */
                => quote! { Time },
            "timestamp"       /* D: 1114, 1115 */
                => quote! { DateTime },
            "timestamptz"     /* D: 1184, 1185 */
                => quote! { DateTimeTz },

            //"box"             /* G:  603, 1020 */
            //"circle"          /* G:  718,  719 */
            //"line"            /* G:  628,  629 */
            //"lseg"            /* G:  601, 1018 */
            //"path"            /* G:  602, 1019 */
            //"point"           /* G:  600, 1017 */
            //"polygon"         /* G:  604, 1027 */

            //"cidr"            /* I:  650,  651 */
            //"inet"            /* I:  869, 1041 */
            //"macaddr"         /* U:  829, 1040 */
            //"macaddr8"        /* U:  774,  775 */

            "float4"          /* N:  700, 1021 */
                => quote! { f32 },
            "float8"          /* N:  701, 1022 */
                => quote! { f64 },
            "int2"            /* N:   21, 1005 */
                => quote! { i16 },
            "int4"            /* N:   23, 1007 */
            | "xid"           /* U:   28, 1011 */
                => quote! { i32 },
            "int8"            /* N:   20, 1016 */
            | "xid8"          /* U: 5069,  271 */
                => quote! { i64 },
            "money"           /* N:  790,  791 */
            | "numeric"       /* N: 1700, 1231 */
                => quote! { Decimal },
            "oid"             /* N:   26, 1028 */
                => quote! { u32 },

            //"regclass"        /* N: 2205, 2210 */
            //"regcollation"    /* N: 4191, 4192 */
            //"regconfig"       /* N: 3734, 3735 */
            //"regdictionary"   /* N: 3769, 3770 */
            //"regnamespace"    /* N: 4089, 4090 */
            //"regoper"         /* N: 2203, 2208 */
            //"regoperator"     /* N: 2204, 2209 */
            //"regproc"         /* N:   24, 1008 */
            //"regprocedure"    /* N: 2202, 2207 */
            //"regrole"         /* N: 4096, 4097 */
            //"regtype"         /* N: 2206, 2211 */

            //"datemultirange"  /* R: 4535, 6155 */
            //"daterange"       /* R: 3912, 3913 */
            //"int4multirange"  /* R: 4451, 6150 */
            //"int4range"       /* R: 3904, 3905 */
            //"int8multirange"  /* R: 4536, 6157 */
            //"int8range"       /* R: 3926, 3927 */
            //"nummultirange"   /* R: 4532, 6151 */
            //"numrange"        /* R: 3906, 3907 */
            //"tsmultirange"    /* R: 4533, 6152 */
            //"tsrange"         /* R: 3908, 3909 */
            //"tstzmultirange"  /* R: 4534, 6153 */
            //"tstzrange"       /* R: 3910, 3911 */

            "bpchar"          /* S: 1042, 1014 */
            | "name"          /* S:   19, 1003 */
            | "text"          /* S:   25, 1009 */
            | "varchar"       /* S: 1043, 1015 */
                => quote! { String },
            //"interval"      /* T: 1186, 1187 */
            //"aclitem"       /* U: 1033, 1034 */
            "bytea"           /* U:   17, 1001 */
                => quote! { Vec<u8> },
            //"cid"           /* U:   29, 1012 */
            "json"            /* U:  114,  199 */
            | "jsonb"          /* U: 3802, 3807 */
                => quote! { Json },
            //"jsonpath"      /* U: 4072, 4073 */

            //"pg_lsn"        /* U: 3220, 3221 */
            //"pg_snapshot"   /* U: 5038, 5039 */
            //"refcursor"     /* U: 1790, 2201 */
            //"tid"           /* U:   27, 1010 */

            //"gtsvector"     /* U: 3642, 3644 */
            //"tsquery"       /* U: 3615, 3645 */
            //"tsvector"      /* U: 3614, 3643 */
            //"txid_snapshot" /* U: 2970, 2949 */
            "uuid"            /* U: 2950, 2951 */
                => quote! { Uuid },

            //"xml"           /* U:  142,  143 */

            //"bit"           /* V: 1560, 1561 */
            //"varbit"        /* V: 1562, 1563 */
            unknown
                => quote! { Unknown<#unknown> },
        };
        let mut full_type = base_type;
        if self.is_array {
            full_type = quote! { Vec<#full_type> };
        }
        if self.is_nullable {
            full_type = quote! { Option<#full_type> };
        }
        full_type
    }
}

const SCHEMA_QUERY: &str = include_str!("reflect/schema.postgres.sql");

#[derive(Parser, Debug)]
pub struct Reflect {
    /// Database schema name
    #[clap(long, default_value = "public")]
    schema: String,
    /// Destination filename [default stdout]
    #[clap(long, short)]
    output: Option<String>,
}

impl Reflect {
    pub fn run(self) -> Result<()> {
        let runtime = create_runtime();
        let url = get_var_database_url();
        let mut conn = runtime.block_on(PgConnection::connect(&url))?;
        let conn = runtime.block_on(conn.acquire())?;

        let schema =
            runtime.block_on(ormlite::query_as::<_, ColumnDef>(SCHEMA_QUERY).bind(self.schema).fetch_all(conn))?;

        let q_models = schema
            .into_iter()
            .group_by(|item| (item.table_schema.to_owned(), item.table_name.to_owned()))
            .into_iter()
            .map(|(key, items)| {
                let table_name_sql = &key.1;
                let table_name_rs = format_ident!("{}", table_name_sql.to_pascal_case());
                let q_table_alias = (table_name_rs != *table_name_sql).then_some(quote! {
                    #[ormlite(table=#table_name_sql)]
                });
                let q_columns = items.into_iter().map(|col| {
                    let q_is_primary_key = col.is_primary_key.then_some(quote! {
                        #[ormlite(primary_key)]
                    });
                    if let (Some(_fk_column_name), Some(fk_table_name)) = (&col.fk_column_name, &col.fk_table_name) {
                        let column_name_sql = &col.column_name.clone();
                        let type_name_rs = format_ident!("{}", fk_table_name.to_pascal_case());
                        if col.is_rev_fk {
                            let coulmn_name_rs = format_ident!("{}", fk_table_name.to_snake_case());
                            quote! {
                                #[ormlite(skip)]
                                pub #coulmn_name_rs: KVec<#type_name_rs>,
                            }
                        } else {
                            let column_name_rs = format_ident!(
                                "{}",
                                col.column_name.strip_prefix("id_").unwrap_or(&col.column_name).to_snake_case()
                            );
                            quote! {
                                #[ormlite(join_column = #column_name_sql)]
                                pub #column_name_rs: Join<#type_name_rs>,
                            }
                        }
                    } else {
                        let column_name_rs = format_ident!("{}", col.column_name.to_snake_case());
                        let column_type_rs = col.rust_type();
                        quote! {
                            #q_is_primary_key
                            pub #column_name_rs: #column_type_rs,
                        }
                    }
                });
                quote! {
                    #[derive(Model, Debug, Serialize)]
                    #q_table_alias
                    struct #table_name_rs {
                        #(#q_columns)*
                    }
                }
            })
            .concat();

        let q_output = quote! {
            use ormlite::{Model, model::Join};
            use serde::Serialize;
            pub use ormlite::types::{
                chrono::{
                    NaiveDate as Date,
                    NaiveDateTime as DateTime,
                    NaiveTime as Time,
                    DateTime as ChronoDateTime,
                    FixedOffset,
                    Local,
                    Utc
                },
                // BigDecimal,
                Decimal,
                Json,
                Uuid
            };
            pub type DateTimeTz = ChronoDateTime<FixedOffset>;
            pub type DateTimeUtc = ChronoDateTime<Utc>;
            pub type DateTimeLocal = ChronoDateTime<Local>;

            /*
            pub use time::Date as Date;
            pub use time::Time as Time;
            pub use time::PrimitiveDateTime as DateTime;
            pub use time::OffsetDateTime as DateTimeTz;
            */

            #q_models
        };

        let output = PrettyPlease::default().format_tokens(q_output).unwrap();
        if let Some(filename) = self.output {
            fs::write(filename, output)?;
        } else {
            println!("{output}");
        }
        eprintln!("{} Reflected database at {}", "SUCCESS".green(), url);
        Ok(())
    }
}
