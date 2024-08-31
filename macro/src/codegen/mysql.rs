use crate::codegen::common::OrmliteCodegen;

use ormlite_core::query_builder::Placeholder;
use proc_macro2::TokenStream;
use quote::quote;

pub struct MysqlBackend {}

impl OrmliteCodegen for MysqlBackend {
    fn database_ts(&self) -> TokenStream {
        quote! { ::ormlite::mysql::Mysql }
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
        quote! { ::ormlite::mysql::MysqlRow }
    }
}
