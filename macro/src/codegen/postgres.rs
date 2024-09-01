use crate::codegen::common::{from_row_bounds, OrmliteCodegen};
use crate::MetadataCache;
use ormlite_core::query_builder::Placeholder;
use proc_macro2::TokenStream;
use quote::quote;

pub struct PostgresBackend;

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

    fn row(&self) -> TokenStream {
        quote! {
            ::ormlite::postgres::PgRow
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ormlite_attr::ttype::InnerType;
    use ormlite_attr::ColumnMeta;
    use ormlite_attr::ModelMeta;

    #[test]
    fn test_all_bounds() {
        let db = PostgresBackend;
        let mut cache = MetadataCache::new();
        let table = ModelMeta::mock(
            "user",
            vec![
                ColumnMeta::mock("id", "u32"),
                ColumnMeta::mock("name", "String"),
                ColumnMeta::mock("organization_id", "u32"),
                ColumnMeta::mock_join("organization", "Organization"),
            ],
        );
        cache.insert("User".to_string(), table.clone());
        let table = ModelMeta::mock(
            "organization",
            vec![
                ColumnMeta::mock("id", "u32"),
                ColumnMeta::mock("name", "String"),
                ColumnMeta::mock("is_active", "bool"),
            ],
        );
        cache.insert("Organization".to_string(), table.clone());

        let types_for_bound = crate::codegen::common::table_primitive_types(&table.table, &cache);
        let types_for_bound = types_for_bound.into_iter().map(|c| c.into_owned()).collect::<Vec<_>>();
        assert_eq!(
            types_for_bound,
            vec![
                InnerType::mock("u32"),
                InnerType::mock("String"),
                InnerType::mock("bool"),
            ]
        );
        let bounds = from_row_bounds(&db, &table.table, &cache);
        let bounds = quote! {
            #(#bounds)*
        };
        assert_eq!(
            bounds.to_string(),
            "u32 : :: ormlite :: decode :: Decode < 'a , R :: Database > , ".to_owned()
                + "u32 : :: ormlite :: types :: Type < R :: Database > , "
                + "String : :: ormlite :: decode :: Decode < 'a , R :: Database > , "
                + "String : :: ormlite :: types :: Type < R :: Database > , "
                + "bool : :: ormlite :: decode :: Decode < 'a , R :: Database > , "
                + "bool : :: ormlite :: types :: Type < R :: Database > ,"
        );
    }
}
