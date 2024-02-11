use proc_macro2::TokenStream;
use quote::quote;
use ormlite_attr::TableMetadata;
use ormlite_attr::Ident;
use ormlite_attr::ColumnMetadata;
use ormlite_attr::ModelMetadata;
use ormlite_attr::TType;
use crate::codegen::common::{generate_conditional_bind, insertion_binding, OrmliteCodegen};
use crate::MetadataCache;

pub fn impl_Model__insert(db: &dyn OrmliteCodegen, attr: &TableMetadata, metadata_cache: &MetadataCache) -> TokenStream {
    let box_future = crate::util::box_fut_ts();
    let mut placeholder = db.placeholder();
    let db = db.database_ts();
    let table = &attr.table_name;
    let params = attr.database_columns()
        .map(|_| placeholder.next().unwrap());

    let query_bindings = attr.database_columns().map(|c| insertion_binding(c));

    let insert_join = attr.many_to_one_joins().map(|c| insert_join(c));

    let late_bind = attr.many_to_one_joins().map(|c| {
        let id = &c.identifier;
        quote! {
            model.#id = #id;
        }
    });

    quote! {
        fn insert<'a, A>(mut self, conn: A) -> ::ormlite::__private::Insertion<'a, A, Self, #db>
            where
                A: 'a + Send + ::ormlite::Acquire<'a, Database=#db>
        {
            ::ormlite::__private::Insertion {
                acquire: conn,
                model: self,
                closure: Box::new(|conn, mut model, query| {
                    Box::pin(async move {
                        let mut conn = conn.acquire().await?;
                        #(
                            #insert_join
                        )*
                        let mut q = ::ormlite::query_as(&query);
                        #(
                            #query_bindings
                        )*
                        // using fetch instead of fetch_one because of https://github.com/launchbadge/sqlx/issues/1370
                        let mut stream = q.fetch(&mut *conn);
                        let mut model: Self = ::ormlite::__private::StreamExt::try_next(&mut stream)
                            .await?
                            .ok_or_else(|| ::ormlite::Error::from(::ormlite::SqlxError::RowNotFound))?;
                        ::ormlite::__private::StreamExt::try_next(&mut stream).await?;
                        #(
                            #late_bind
                        )*
                        Ok(model)
                    })
                }),
                insert: ::ormlite::__private::Insert::new(#table)
                    .columns(<Self as ::ormlite::TableMeta>::table_columns())
                    .one_value(&[#(#params,)*])
                    .returning(<Self as ::ormlite::TableMeta>::table_columns()),
                _db: ::std::marker::PhantomData,
            }
        }
    }
}


pub fn impl_ModelBuilder__insert(db: &dyn OrmliteCodegen, attr: &TableMetadata) -> TokenStream {
    let box_future = crate::util::box_fut_ts();
    let placeholder = db.placeholder_ts();
    let db = db.database_ts();
    let query = format!(
        "INSERT INTO \"{}\" ({{}}) VALUES ({{}}) RETURNING *",
        attr.table_name
    );

    let bind_parameters = attr.database_columns().map(generate_conditional_bind);

    quote! {
        fn insert<'e: 'a, E>(self, db: E) -> #box_future<'a, ::ormlite::Result<Self::Model>>
        where
            E: 'e +::ormlite::Executor<'e, Database = #db>,
        {
            Box::pin(async move {
                let mut placeholder = #placeholder;
                let set_fields = self.modified_fields();
                let query = format!(
                    #query,
                    set_fields.join(", "),
                    set_fields.iter().map(|_| placeholder.next().unwrap()).collect::<Vec<_>>().join(", "),
                );
                let mut q = ::ormlite::query_as::<#db, Self::Model>(&query);
                #(#bind_parameters)*
                // using fetch instead of fetch_one because of https://github.com/launchbadge/sqlx/issues/1370
                let mut stream = q.fetch(db);
                let model = ::ormlite::__private::StreamExt::try_next(&mut stream)
                    .await?
                    .ok_or_else(|| ::ormlite::Error::from(::ormlite::SqlxError::RowNotFound))?;
                ::ormlite::__private::StreamExt::try_next(&mut stream).await?;
                Ok(model)
            })
        }
    }
}

pub fn impl_InsertModel(db: &dyn OrmliteCodegen, meta: &ModelMetadata) -> TokenStream {
    let Some(insert_struct) = &meta.insert_struct else {
        return quote! {};
    };
    let box_future = crate::util::box_fut_ts();
    let model = &meta.inner.struct_name;
    let mut placeholder = db.placeholder();
    let db = db.database_ts();
    let insert_model = Ident::new(&insert_struct);
    let fields = meta.database_columns()
        .filter(|&c| !c.has_database_default)
        .map(|c| c.column_name.clone())
        .collect::<Vec<_>>()
        .join(",");
    let placeholders = meta.database_columns()
        .filter(|&c| !c.has_database_default)
        .map(|_| placeholder.next().unwrap())
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING *",
        meta.inner.table_name, fields, placeholders,
    );

    let query_bindings = meta.database_columns()
        .filter(|&c| !c.has_database_default)
        .map(|c| {
            if let Some(rust_default) = &c.rust_default {
                let default: syn::Expr = syn::parse_str(&rust_default).expect("Failed to parse default_value");
                return quote! {
                    q = q.bind(#default);
                }
            }
            insertion_binding(c)
        });

    let insert_join = meta.inner.many_to_one_joins().map(|c| insert_join(c));

    let late_bind = meta.many_to_one_joins().map(|c| {
        let id = &c.identifier;
        quote! {
            model.#id = #id;
        }
    });

    quote! {
        impl ::ormlite::model::Insertable<#db> for #insert_model {
            type Model = #model;

            fn insert<'a, A>(self, db: A) -> #box_future<'a, ::ormlite::Result<Self::Model>>
            where
                A: 'a + Send + ::ormlite::Acquire<'a, Database = #db>,
            {
                Box::pin(async move {
                    let mut conn = db.acquire().await?;
                    let mut q =::ormlite::query_as::<#db, Self::Model>(#query);
                    let mut model = self;
                    #(#insert_join)*
                    #(#query_bindings)*
                    let mut stream = q.fetch(&mut *conn);
                    let mut model = ::ormlite::__private::StreamExt::try_next(&mut stream)
                        .await?
                        .ok_or_else(|| ::ormlite::Error::from(::ormlite::SqlxError::RowNotFound))?;
                    ::ormlite::__private::StreamExt::try_next(&mut stream).await?;
                    #(#late_bind)*
                    Ok(model)
                })
            }
        }
    }
}


/// Insert joined structs
/// Assumed bindings:
/// - `model`: model struct
/// - `#id`: Other code relies on this binding being created
pub fn insert_join(c: &ColumnMetadata) -> TokenStream {
    let id = &c.identifier;
    let joined_ty = c.column_type.joined_type().unwrap();

    let preexisting = match joined_ty {
        TType::Option(joined_ty) => {
            quote! {
                if let Some(id) = model.#id._id() {
                    #joined_ty::fetch_one(id, &mut *conn).await?
                } else {
                    None
                }
            }
        }
        joined_ty => {
            quote! {
                #joined_ty::fetch_one(model.#id._id(), &mut *conn).await?
            }
        }
    };

    quote! {
        let #id = if let Some(modification) = model.#id._take_modification() {
            match modification
                    .insert(&mut *conn)
                    .on_conflict(::ormlite::query_builder::OnConflict::Ignore)
                    .await {
                Ok(model) => Join::_query_result(model),
                Err(::ormlite::Error::SqlxError(::ormlite::SqlxError::RowNotFound)) => {
                    let preexisting = #preexisting;
                    Join::_query_result(preexisting)
                },
                Err(e) => return Err(e),
            }
        } else {
            model.#id
        };
    }
}
