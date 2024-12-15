use crate::codegen::common::{from_row_bounds, OrmliteCodegen};
use crate::MetadataCache;
use ormlite_attr::Ident;
use ormlite_attr::TableMeta;
use ormlite_attr::{ColumnMeta, Type};
use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_FromRow(db: &dyn OrmliteCodegen, attr: &TableMeta, cache: &MetadataCache) -> TokenStream {
    let bounds = from_row_bounds(db, attr, cache);
    let row = db.row();

    let prefix_branches = attr.columns.iter().filter(|c| c.is_join()).map(|c| {
        let name = &c.ident.to_string();
        let iden = &c.ident;
        let meta = cache
            .get(c.joined_struct_name().unwrap().as_str())
            .expect("Joined struct not found");
        let result = if c.is_join_many() {
            unimplemented!("Join<Vec<...>> isn't supported quite yet...");
        } else {
            let prefixed_columns = meta.database_columns().map(|c| format!("__{}__{}", iden, c.ident));
            let path = c.joined_model();
            quote! {
                #path::from_row_using_aliases(row, &[
                    #(
                        #prefixed_columns,
                    )*
                ])?
            }
        };
        quote! {
            #name => {
                model.#iden = ::ormlite::model::Join::_query_result(#result);
            }
        }
    });

    let field_names = attr.database_columns().map(|c| &c.name);

    let map_join = if attr.columns.iter().any(|c| c.is_join()) {
        quote! {
            let mut prefixes = ::ormlite::Row::columns(row).iter().filter_map(|c| {
                let name = ::ormlite::Column::name(c);
                if name.starts_with("__") {
                    name.rsplitn(2, "__").last().map(|s| &s[2..])
                } else {
                    None
                }
            })
                .collect::<Vec<_>>();
            prefixes.sort();
            prefixes.dedup();
            for prefix in prefixes {
                match prefix {
                    #(
                        #prefix_branches
                    )*
                    _ => {
                        return Err(::ormlite::SqlxError::Decode(
                            Box::new(::ormlite::CoreError::OrmliteError(format!("Unknown column prefix: {}", prefix))),
                        ));
                    }
                }
            }
        }
    } else {
        TokenStream::new()
    };
    let model = &attr.ident;
    quote! {
        impl<'a> ::ormlite::model::FromRow<'a, #row> for #model
            where
                // &'a str: ::ormlite::ColumnIndex<#row>,
                #(
                    #bounds
                )*
        {
            fn from_row(row: &'a #row) -> ::std::result::Result<Self, ::ormlite::SqlxError> {
                let mut model = Self::from_row_using_aliases(row, &[
                    #(
                        #field_names,
                    )*
                ])?;
                #map_join
                Ok(model)
            }
        }
    }
}

pub fn impl_from_row_using_aliases(
    db: &dyn OrmliteCodegen,
    attr: &TableMeta,
    metadata_cache: &MetadataCache,
) -> TokenStream {
    let row = db.row();
    let fields = attr.all_fields();
    let bounds = from_row_bounds(db, attr, &metadata_cache);
    let mut incrementer = 0usize..;
    let columns = attr
        .columns
        .iter()
        .map(|c| {
            let index = incrementer.next().unwrap();
            let get = quote! { aliases[#index] };
            from_row_for_column(get, c)
        })
        .collect::<Vec<_>>();

    let model = &attr.ident;
    quote! {
        impl #model {
            pub fn from_row_using_aliases<'a>(row: &'a #row, aliases: &'a [&str]) -> ::std::result::Result<Self, ::ormlite::SqlxError>
                where
                    #(
                        #bounds
                    )*
            {
                #(
                    #columns
                )*
                Ok(Self { #(#fields,)* })
            }
        }
    }
}

/// `name` renames the column. Can pass `col.column_name` if it's not renamed.
pub fn from_row_for_column(get_value: TokenStream, col: &ColumnMeta) -> TokenStream {
    let id = &col.ident;
    let ty = &col.ty;
    if col.skip {
        quote! {
            let #id = Default::default();
        }
    } else if col.is_join() {
        let id_id = Ident::from(format!("{}_id", id));
        quote! {
            let #id_id: <#ty as ::ormlite::model::JoinMeta>::IdType = ::ormlite::Row::try_get(row, #get_value)?;
            let #id = ::ormlite::model::Join::new_with_id(#id_id);
        }
    } else if col.json {
        if let Type::Option(inner) = ty {
            quote! {
                let #id: Option<::ormlite::types::Json<#inner>> = ::ormlite::Row::try_get(row, #get_value)?;
                let #id = #id.map(|j| j.0);
            }
        } else {
            quote! {
                let #id: ::ormlite::types::Json<#ty> = ::ormlite::Row::try_get(row, #get_value)?;
                let #id = #id.0;
            }
        }
    } else {
        quote! {
            let #id: #ty = ::ormlite::Row::try_get(row, #get_value)?;
        }
    }
}
