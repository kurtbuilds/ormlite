use crate::codegen::common::OrmliteCodegen;

use proc_macro2::TokenStream;
use quote::quote;

pub struct SqliteBackend {}

impl OrmliteCodegen for SqliteBackend {
    fn database() -> TokenStream {
        quote! { ::sqlx::Sqlite }
    }

    fn placeholder() -> TokenStream {
        quote! {
            std::iter::repeat("?".to_string())
        }
    }

    fn raw_placeholder() -> Box<dyn Iterator<Item = String>> {
        Box::new(std::iter::repeat("?".to_string()))
    }
}
