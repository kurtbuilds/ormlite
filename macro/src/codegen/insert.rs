use itertools::Itertools;
use crate::codegen::common::{generate_conditional_bind, insertion_binding, OrmliteCodegen};
use crate::MetadataCache;
use ormlite_attr::ColumnMeta;
use ormlite_attr::Ident;
use ormlite_attr::ModelMeta;
use ormlite_attr::TableMeta;
use ormlite_attr::Type;
use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_Model__insert(db: &dyn OrmliteCodegen, attr: &ModelMeta, metadata_cache: &MetadataCache) -> TokenStream {
    let box_future = crate::util::box_fut_ts();
    let mut placeholder = db.placeholder();
    let db = db.database_ts();
    let table = &attr.name;
    let params = attr.database_columns().map(|c| {
        if attr.pkey.name == c.name {
            placeholder.next().unwrap()
        } else if c.has_database_default {
            "DEFAULT".to_string()
        } else {
            placeholder.next().unwrap()
        }
    });

    let query_bindings = attr
        .database_columns()
        .filter(|c| attr.pkey.name == c.name || !c.has_database_default)
        .map(|c| insertion_binding(c));

    let insert_join = attr.many_to_one_joins().map(|c| insert_join(c));

    let late_bind = attr.many_to_one_joins().map(|c| {
        let id = &c.ident;
        quote! {
            model.#id = #id;
        }
    });

    quote! {
        #[allow(unused_mut)]
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
                        let mut model: Self = q.fetch_one(&mut *conn).await?;
                        #(
                            #late_bind
                        )*
                        Ok(model)
                    })
                }),
                insert: ::ormlite::__private::Insert::new(#table)
                    .columns(<Self as ::ormlite::TableMeta>::table_columns())
                    .values(::ormlite::__private::Values::from([#(#params,)*].as_slice()))
                    .returning(<Self as ::ormlite::TableMeta>::table_columns()),
                _db: ::std::marker::PhantomData,
            }
        }
    }
}

pub fn impl_ModelBuilder__insert(db: &dyn OrmliteCodegen, attr: &TableMeta) -> TokenStream {
    let box_future = crate::util::box_fut_ts();
    let placeholder = db.placeholder_ts();
    let db = db.database_ts();
    let query = format!("INSERT INTO \"{}\" ({{}}) VALUES ({{}}) RETURNING *", attr.name);

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
                let model = q.fetch_one(db).await?;
                Ok(model)
            })
        }
    }
}

pub fn impl_InsertModel(db: &dyn OrmliteCodegen, meta: &ModelMeta) -> TokenStream {
    let Some(insert_struct) = &meta.insert_struct else {
        return TokenStream::new();
    };
    impl_Insert(db, meta, insert_struct, &meta.ident)
}

pub fn impl_Insert(db: &dyn OrmliteCodegen, meta: &TableMeta, model: &Ident, returns: &Ident) -> TokenStream {
    let box_future = crate::util::box_fut_ts();
    let mut placeholder = db.placeholder();
    let db = db.database_ts();
    let fields = meta
        .database_columns()
        .filter(|&c| !c.has_database_default)
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
        .join(",");
    let placeholders = meta
        .database_columns()
        .filter(|&c| !c.has_database_default)
        .map(|_| placeholder.next().unwrap())
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING *",
        meta.name, fields, placeholders,
    );
    let query_bindings = meta.database_columns().filter(|&c| !c.has_database_default).map(|c| {
        if let Some(rust_default) = &c.rust_default {
            let default: syn::Expr = syn::parse_str(&rust_default).expect("Failed to parse default_value");
            quote! {
                q = q.bind(#default);
            }
        } else {
            insertion_binding(c)
        }
    });

    let insert_join = meta.many_to_one_joins().map(|c| insert_join(c));

    let late_bind = meta.many_to_one_joins().map(|c| {
        let id = &c.ident;
        quote! {
            model.#id = #id;
        }
    });

    quote! {
        impl ::ormlite::model::Insert<#db> for #model {
            type Model = #returns;

            #[allow(unused_mut)]
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
                    let mut model: #returns = q.fetch_one(&mut *conn).await?;
                    #(#late_bind)*
                    ::ormlite::Result::<Self::Model>::Ok(model)
                })
            }
        }
    }
}

/// Insert joined structs
/// Assumed bindings:
/// - `model`: model struct
/// - `#id`: Other code relies on this binding being created
pub fn insert_join(c: &ColumnMeta) -> TokenStream {
    let id = &c.ident;
    let joined_ty = c.ty.joined_type().unwrap();

    let preexisting = match joined_ty {
        Type::Option(joined_ty) => {
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

pub fn impl_Model__insert_many(db: &dyn OrmliteCodegen, meta: &ModelMeta, _mc: &MetadataCache) -> TokenStream {
    let box_future = crate::util::box_fut_ts();
    let placeholder = db.placeholder_ts();
    let dialect = db.dialect_ts();
    let db = db.database_ts();

    let query_bindings = meta.database_columns().map(|c| {
        if let Some(rust_default) = &c.rust_default {
            let default: syn::Expr = syn::parse_str(&rust_default).expect("Failed to parse default_value");
            quote! {
                q = q.bind(#default);
            }
        } else if c.is_join() {
            let name = &c.ident;
            quote! {
                q = q.bind(model.#name._id());
            }
        } else {
            insertion_binding(c)
        }
    }).collect_vec();

    quote! {
        fn insert_many<'e, E>(values: Vec<Self>, db: E) -> #box_future<'e, ::ormlite::Result<Vec<Self>>>
        where
            E: 'e + ::ormlite::Executor<'e, Database = #db>,
        {
            Box::pin(async move {
                let table = <Self as ::ormlite::TableMeta>::table_name();
                let columns = <Self as ::ormlite::TableMeta>::table_columns();
                let mut sql_values = ::ormlite::__private::Values::Values(Vec::new());
                for _ in 0..values.len() {
                    let mut value = ::ormlite::__private::Value::new();
                    value = value.placeholders(columns.len(), #dialect);
                    sql_values = sql_values.value(value);
                }
                let sql = ::ormlite::__private::Insert::new(table)
                    .columns(columns)
                    .values(sql_values)
                    .returning(columns);
                let sql = ::ormlite::__private::ToSql::to_sql(&sql, #dialect);
                let mut q = ::ormlite::query_as::<#db, Self>(&sql);
                for model in values {
                    #(#query_bindings)*
                }
                q.fetch_all(db).await.map_err(Into::into)
            })
        }
    }
}