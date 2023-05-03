use proc_macro2::TokenStream;
use quote::quote;
use ormlite_attr::TableMetadata;
use crate::codegen::common::OrmliteCodegen;

pub fn impl_Model__select(db: &dyn OrmliteCodegen, attr: &TableMetadata) -> TokenStream {
    let table_name = &attr.table_name;
    let db = db.database_ts();
    quote! {
        fn select<'args>() -> ::ormlite::query_builder::SelectQueryBuilder<'args, #db, Self> {
            ::ormlite::query_builder::SelectQueryBuilder::default()
                .select(format!("\"{}\".*", #table_name))
        }
    }
}