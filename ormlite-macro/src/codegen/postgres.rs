use crate::codegen::common::OrmliteCodegen;
use proc_macro2::TokenStream;
use quote::quote;

pub struct PostgresBackend {}

impl OrmliteCodegen for PostgresBackend {
    fn database() -> TokenStream {
        quote! { sqlx::Postgres }
    }

    fn placeholder() -> TokenStream {
        quote! {
            (0..u32::MAX).map(|i| format!("${}", i))
        }
    }

    fn raw_placeholder() -> Box<dyn Iterator<Item = String>> {
        Box::new((1..u32::MAX).map(|i| format!("${}", i)))
    }
}
