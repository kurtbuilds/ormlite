use crate::codegen::common::OrmliteCodegen;
use crate::TableMeta;
use ormlite_core::query_builder::Placeholder;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub struct PostgresBackend {}

impl OrmliteCodegen for PostgresBackend {
    fn database() -> TokenStream {
        quote! { sqlx::Postgres }
    }

    fn placeholder() -> TokenStream {
        quote! {
            ::ormlite::query_builder::Placeholder::dollar_sign(0)
        }
    }

    fn raw_placeholder() -> Placeholder {
        Placeholder::dollar_sign(0)
    }

    fn impl_HasQueryBuilder(ast: &DeriveInput, attr: &TableMeta) -> TokenStream {
        let table_name = &attr.table_name;
        let model = &ast.ident;
        let db = Self::database();
        quote! {
            impl ::ormlite::model::HasQueryBuilder<#db> for #model {
                fn select<'args>() -> ::ormlite::SelectQueryBuilder<'args, #db, Self> {
                    ::ormlite::SelectQueryBuilder::new(::ormlite::query_builder::Placeholder::dollar_sign(0))
                        .column(&format!("{}.*", #table_name))
                }
            }
        }
    }
}
