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
            ::ormlite::query_builder::Placeholder::dollar_sign()
        }
    }

    fn raw_placeholder() -> Placeholder {
        Placeholder::dollar_sign()
    }

    fn impl_Model__select(_ast: &DeriveInput, attr: &TableMeta) -> TokenStream {
        let table_name = &attr.table_name;
        let db = Self::database();
        quote! {
            fn select<'args>() -> ::ormlite::SelectQueryBuilder<'args, #db, Self> {
                ::ormlite::SelectQueryBuilder::new(::ormlite::query_builder::Placeholder::dollar_sign())
                    .column(&format!("\"{}\".*", #table_name))
            }
        }
    }
}
