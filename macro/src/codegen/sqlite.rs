use crate::codegen::common::OrmliteCodegen;

use ormlite_core::query_builder::Placeholder;
use proc_macro2::TokenStream;
use quote::quote;

pub struct SqliteBackend {}

impl OrmliteCodegen for SqliteBackend {
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
}
