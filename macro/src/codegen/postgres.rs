use crate::codegen::common::OrmliteCodegen;
use ormlite_attr::TableMetadata;
use ormlite_core::query_builder::Placeholder;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub struct PostgresBackend {}

impl OrmliteCodegen for PostgresBackend {
    fn database_ts(&self) -> TokenStream {
        quote! { ::ormlite::postgres::Postgres }
    }

    fn placeholder_ts(&self) -> TokenStream {
        quote! {
            ::ormlite::query_builder::Placeholder::dollar_sign()
        }
    }

    fn placeholder(&self) -> Placeholder {
        Placeholder::dollar_sign()
    }

}
