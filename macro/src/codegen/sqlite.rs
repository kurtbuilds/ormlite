use crate::codegen::common::OrmliteCodegen;

use ormlite_core::query_builder::Placeholder;
use proc_macro2::TokenStream;
use quote::quote;

pub struct SqliteBackend {}

impl OrmliteCodegen for SqliteBackend {
    fn database() -> TokenStream {
        quote! { ::sqlx::Sqlite }
    }

    fn placeholder() -> TokenStream {
        quote! {
            ::ormlite::query_builder::Placeholder::question_mark()
        }
    }

    fn raw_placeholder() -> Placeholder {
        Placeholder::question_mark()
    }
}
