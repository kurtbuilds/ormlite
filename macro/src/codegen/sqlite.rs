use crate::codegen::common::OrmliteCodegen;

use ormlite_core::query_builder::Placeholder;
use proc_macro2::TokenStream;
use quote::quote;

pub struct SqliteBackend {}

impl OrmliteCodegen for SqliteBackend {
    fn dialect_ts(&self) -> TokenStream {
        quote! { ::ormlite::__private::Dialect::Sqlite }
    }
    fn database_ts(&self) -> TokenStream {
        quote! { ::ormlite::sqlite::Sqlite }
    }

    fn placeholder_ts(&self) -> TokenStream {
        quote! {
            ::ormlite::query_builder::Placeholder::question_mark()
        }
    }

    fn placeholder(&self) -> Placeholder {
        Placeholder::question_mark()
    }

    fn row(&self) -> TokenStream {
        quote! {
            ::ormlite::sqlite::SqliteRow
        }
    }
}
