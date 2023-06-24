
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use ormlite_attr::{Ident, TableMetadata};
use crate::MetadataCache;

pub fn static_join_descriptions(attr: &TableMetadata, metadata_cache: &MetadataCache) -> TokenStream {
    let model = &attr.struct_name;
    let joins = attr.many_to_one_joins().map(|c| {
        let id = &c.identifier;
        let id_s = &c.identifier.to_string();
        let struct_name = c.joined_struct_name().unwrap();
        let joined_table = metadata_cache.get(&struct_name).expect(&format!("Did not find metadata for joined struct: {}", struct_name));

        let column_name = c.many_to_one_column_name.as_ref().unwrap();
        let foreign_key = &joined_table.pkey.column_name;
        let join_type = if c.one_to_many_foreign_key.is_some() {
            quote! { ::ormlite::__private::SemanticJoinType::OneToMany }
        } else if c.many_to_one_column_name.is_some() {
            quote! { ::ormlite::__private::SemanticJoinType::ManyToOne }
        } else if c.many_to_many_table.is_some() {
            quote! { ::ormlite::__private::SemanticJoinType::ManyToMany }
        } else {
            panic!("Unknown join type");
        };

        let table_schema = &joined_table.schema_name.clone().unwrap_or_default();
        let table_name = &joined_table.table_name;
        let columns = joined_table
            .columns
            .iter()
            .filter(|c| !c.is_join())
            .map(|c| c.identifier.to_string());
        quote! {
            pub fn #id() -> ::ormlite::__private::JoinDescription {
                ::ormlite::__private::JoinDescription {
                    joined_columns: &[
                        #(
                            #columns,
                        )*
                    ],
                    table_schema: if #table_schema.is_empty() {
                        None
                    } else {
                        Some(#table_schema)
                    },
                    table_name: #table_name,
                    relation: #id_s,
                    key: #column_name,
                    foreign_key: #foreign_key,
                    semantic_join_type: #join_type,
                }
            }
        }
    });

    quote! {
        impl #model {
            #(
                #joins
            )*
        }
    }
}