use std::collections::HashMap;
use ormlite_attr as attr;
use ormlite_core::query_builder::Placeholder;
use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Token};
use syn::{DeriveInput, Field};
use syn::spanned::Spanned;
use attr::TableMetadata;
use ormlite_attr::{DeriveInputExt, FieldExt, Type};
use crate::MetadataCache;
use itertools::Itertools;

/// Given the fields of a ModelBuilder struct, return the quoted code.
fn generate_query_binding_code_for_partial_model(
    attr: &TableMetadata,
) -> impl Iterator<Item=TokenStream> + '_ {
    attr.columns.iter()
        .filter(|c| !c.is_join())
        .map(|c| {
            let name = &c.identifier;
            quote! {
                if let Some(value) = self.#name {
                    q = q.bind(value);
                }
            }
        })
}

/// bool whether the given type is `String`
fn ty_is_string(ty: &syn::Type) -> bool {
    let p = match ty {
        syn::Type::Path(p) => p,
        _ => return false,
    };
    p.path.segments.last().map(|s| s.ident == "String").unwrap_or(false)
}


/// `name` renames the column. Can pass `col.column_name` if it's not renamed.
fn from_row_for_column(get_value: proc_macro2::TokenStream, col: &attr::ColumnMetadata) -> proc_macro2::TokenStream {
    let id = &col.identifier;
    let ty = &col.column_type;
    if col.is_join() {
        quote! {
            let #id = ::ormlite::model::Join::default();
        }
    } else {
        quote! {
            let #id: #ty = row.try_get(#get_value)?;
        }
    }
}

fn recursive_primitive_types_ty<'a>(ty: &'a attr::Type, cache: &'a MetadataCache) -> Vec<&'a attr::OtherType> {
    match ty {
        attr::Type::Option(ty) | attr::Type::Vec(ty) => {
            recursive_primitive_types_ty(ty, cache)
        }
        attr::Type::Primitive(p) => vec![p],
        attr::Type::Foreign(_) => {
            panic!("Type::Foreign should not be in a table's columns. Wrap it in `ormlite::postgres::Json`?")
        }
        attr::Type::Join(j) => {
            let joined = cache.get(&j.inner_type_name()).expect("Join type not found");
            recursive_primitive_types(joined, cache)
        }
    }
}

fn recursive_primitive_types<'a>(table: &'a attr::TableMetadata, cache: &'a MetadataCache) -> Vec<&'a attr::OtherType> {
    table.columns.iter()
        .map(|c| {
            recursive_primitive_types_ty(&c.column_type, cache)
        })
        .flatten()
        .collect()
}

fn table_primitive_types<'a>(attr: &'a TableMetadata, cache: &'a MetadataCache) -> Vec<&'a attr::OtherType> {
    attr.columns.iter()
        .map(|c| recursive_primitive_types_ty(&c.column_type, cache))
        .flatten()
        .unique()
        .collect()
}

fn from_row_bounds<'a>(attr: &'a TableMetadata, cache: &'a MetadataCache) -> impl Iterator<Item=proc_macro2::TokenStream> + 'a {
    table_primitive_types(attr, cache)
        .into_iter()
        .map(|ty| {
            quote! {
                #ty: ::ormlite::decode::Decode<'a, R::Database>,
                #ty: ::ormlite::types::Type<R::Database>,
            }
        })
}

fn is_vec(p: &syn::Path) -> bool {
    let Some(segment) = p.segments.last() else {
        return false;
    };
    segment.ident == "Vec"
}

pub trait OrmliteCodegen {
    fn database(&self) -> TokenStream;
    fn placeholder(&self) -> TokenStream;
    fn raw_placeholder(&self) -> Placeholder;

    fn impl_FromRow(&self, ast: &DeriveInput, attr: &attr::TableMetadata, cache: &MetadataCache) -> TokenStream {
        let span = ast.span();
        let model = &ast.ident;

        let bounds = from_row_bounds(attr, cache);

        let columns = attr.columns.iter()
            .map(|c| {
                let name = &c.column_name;
                from_row_for_column(quote! { #name }, c)
            })
            .collect::<Vec<_>>()
            ;
        let fields = attr.all_fields(span);

        let prefix_branches = attr.columns.iter().filter(|c| c.is_join()).map(|c| {
            let name = &c.identifier.to_string();
            let iden = &c.identifier;
            let meta = cache.get(c.joined_struct_name().unwrap().as_str())
                .expect("Joined struct not found");
            let relation = &c.identifier;
            let result = if c.is_join_many() {
                unimplemented!("Join<Vec<...>> isn't supported quite yet...");
            } else {
                let prefixed_columns = meta.columns.iter().map(|c| {
                    format!("__{}__{}", relation, c.identifier)
                });
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

        let field_names = attr.columns.iter()
            .filter(|c| !c.is_join())
            .map(|c| c.identifier.to_string());

        quote! {
            impl<'a, R: ::ormlite::Row> ::ormlite::model::FromRow<'a, R> for #model
                where
                    &'a str: ::ormlite::ColumnIndex<R>,
                    #(
                        #bounds
                    )*
            {
                fn from_row(row: &'a R) -> ::std::result::Result<Self, ::ormlite::SqlxError> {
                    let mut model = Self::from_row_using_aliases(row, &[
                        #(
                            #field_names,
                        )*
                    ])?;
                    let mut prefixes = row.columns().iter().filter_map(|c| {
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
                                    Box::new(::ormlite::Error::OrmliteError(format!("Unknown column prefix: {}", prefix))),
                                ));
                            }
                        }
                    }
                    Ok(model)
                }
            }
        }
    }

    fn impl_from_row_using_aliases(&self, ast: &DeriveInput, attr: &TableMetadata, metadata_cache: &MetadataCache) -> TokenStream {
        let span = ast.span();
        let model = &ast.ident;
        let fields = attr.all_fields(span);
        let bounds = from_row_bounds(attr, &metadata_cache);
        let mut incrementer = 0usize..;
        let columns = attr.columns.iter()
            .map(|c| {
                if c.is_join() {
                    from_row_for_column(proc_macro2::TokenStream::new(), c)
                } else {
                    let index = incrementer.next().unwrap();
                    let get = quote! { aliases[#index] };
                    from_row_for_column(get, c)
                }
            })
            .collect::<Vec<_>>()
            ;

        quote! {
            impl #model {
                pub fn from_row_using_aliases<'a, R: ::ormlite::Row>(row: &'a R, aliases: &'a [&str]) -> ::std::result::Result<Self, ::ormlite::SqlxError>
                    where
                        &'a str: ::ormlite::ColumnIndex<R>,
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

    fn static_join_descriptions(&self, ast: &DeriveInput, attr: &TableMetadata, metadata_cache: &MetadataCache) -> TokenStream {
        let span = ast.span();
        let model = &ast.ident;
        let joins = attr.columns.iter().filter(|c| c.is_join()).map(|c| {
            let relation = &c.identifier;
            let relation_name = &c.identifier.to_string();
            let key = if let Some(key) = &c.many_to_one_key {
                key.to_string()
            } else {
                attr.primary_key.clone().expect("Must have a primary key for the join.")
            };
            let foreign_key = if let Some(foreign_key) = &c.one_to_many_foreign_key {
                unimplemented!("one_to_many_foreign_key not implemented yet");
            } else {
                attr.primary_key.clone().expect("Must have a primary key for the join.")
            };
            let join_type = if c.one_to_many_foreign_key.is_some() {
                quote! { ::ormlite::__private::SemanticJoinType::OneToMany }
            } else if c.many_to_one_key.is_some() {
                quote! { ::ormlite::__private::SemanticJoinType::ManyToOne }
            } else if c.many_to_many_table.is_some() {
                quote! { ::ormlite::__private::SemanticJoinType::ManyToMany }
            } else {
                panic!("Unknown join type");
            };
            let struct_name = c.joined_struct_name().unwrap();
            let joined_table = metadata_cache.get(&struct_name).expect(&format!("Did not find metadata for joined struct: {}", struct_name));
            let table_name = &joined_table.table_name;
            let columns = joined_table.columns.iter().filter(|c| !c.is_join()).map(|c| {
                c.identifier.to_string()
            });
            quote! {
                pub fn #relation() -> ::ormlite::__private::JoinDescription {
                    ::ormlite::__private::JoinDescription {
                        joined_columns: &[
                            #(
                                #columns,
                            )*
                        ],
                        table_name: #table_name,
                        relation: #relation_name,
                        key: #key,
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

    fn impl_Model(&self, ast: &DeriveInput, attr: &attr::TableMetadata, metadata_cache: &MetadataCache) -> TokenStream {
        let db = self.database();
        let model = &ast.ident;
        let partial_model = quote::format_ident!("{}Builder", model.to_string());

        let impl_Model__insert = self.impl_Model__insert(attr, metadata_cache);
        let impl_Model__update_all_fields = self.impl_Model__update_all_fields(ast, attr);
        let impl_Model__delete = self.impl_Model__delete(ast, attr);
        let impl_Model__fetch_one = self.impl_Model__fetch_one(ast, attr);
        let impl_Model__select = self.impl_Model__select(ast, attr);
        let impl_Model__builder = self.impl_Model__builder(ast, attr);
        let impl_Model__update_partial = self.impl_Model__update_partial(ast, attr);

        quote! {
            impl<'slf> ::ormlite::model::Model<'slf, #db> for #model {
                type ModelBuilder = #partial_model<'slf>;

                #impl_Model__insert
                #impl_Model__update_all_fields
                #impl_Model__delete
                #impl_Model__fetch_one
                #impl_Model__select
                #impl_Model__builder
                #impl_Model__update_partial

               fn query(query: &str) -> ::ormlite::query::QueryAs<#db, Self, <#db as ::ormlite::database::HasArguments>::Arguments> {
                    ::ormlite::query_as::<_, Self>(query)
                }
            }
        }
    }

    fn impl_TableMeta(&self, attr: &attr::TableMetadata) -> TokenStream {
        let model = &attr.struct_name;
        let table_name = &attr.table_name;
        let id = match &attr.primary_key {
            Some(id) => quote! { Some(#id) },
            None => quote! { None },
        };
        let field_names = attr.columns.iter()
            .filter(|c| !c.is_join())
            .map(|c| c.column_name.to_string());

        quote! {
            impl ::ormlite::model::TableMeta for #model {
                fn table_name() -> &'static str {
                    #table_name
                }

                fn table_columns() -> &'static [&'static str] {
                    &[#(#field_names,)*]
                }

                fn primary_key() -> Option<&'static str> {
                    #id
                }
            }
        }
    }

    fn impl_IntoArguments(&self, attr: &attr::TableMetadata) -> TokenStream {
        let db = self.database();
        let model = &attr.struct_name;
        let mut placeholder = self.raw_placeholder();
        let params = attr.columns.iter()
            .filter(|c| !c.is_join())
            .map(|c| {
                let field = &c.identifier;
                let value = if c.is_primitive() {
                    quote! { self.#field }
                } else {
                    quote! { ::ormlite::types::Json(self.#field) }
                };
                quote! { ::ormlite::Arguments::add(&mut args, #value); }
            });

        quote! {
            impl<'a> ::ormlite::IntoArguments<'a, #db> for #model {
                fn into_arguments(self) -> <#db as ::ormlite::database::HasArguments<'a>>::Arguments {
                    let mut args = <#db as ::ormlite::database::HasArguments<'a>>::Arguments::default();
                    #(
                        #params
                    )*
                    args
                }
            }
        }
    }

    fn impl_Model__insert(&self, attr: &attr::TableMetadata, metadata_cache: &MetadataCache) -> TokenStream {
        let box_future = crate::util::box_future();
        let db = self.database();
        let table = &attr.table_name;
        let mut placeholder = self.raw_placeholder();
        let params = attr.columns.iter()
            .filter(|c| !c.is_join())
            .map(|_| placeholder.next().unwrap());

        let query_bindings = attr.columns.iter().filter(|c| !c.is_join()).map(|c| {
            let name = &c.identifier;
            quote! {
                q = q.bind(model.#name);
            }
        });

        let rebind = attr.columns.iter().filter(|c| c.is_join() && c.many_to_one_key.is_some()).map(|c| {
            let name = &c.identifier;
            let column = c.many_to_one_key.as_ref().unwrap();
            let joined_struct = c.joined_struct_name().unwrap();
            let joined_meta = metadata_cache.get(&joined_struct).unwrap_or_else(|| panic!("On {}, tried to define join, but failed to find a Model for {}", attr.struct_name, joined_struct));
            let joined_pkey = joined_meta.primary_key.as_ref().expect("Joined struct must have a primary key");
            let joined_pkey = syn::Ident::new(joined_pkey, Span::call_site());
            quote! {
                if let Some(modification) = model.#name._take_modification() {
                    model.#column = modification.#joined_pkey;
                    match modification
                        .insert(&mut *conn)
                        .on_conflict(::ormlite::query_builder::OnConflict::Ignore)
                        .await {
                        Ok(_) => (),
                        Err(::ormlite::Error::SqlxError(::ormlite::SqlxError::RowNotFound)) => (),
                        Err(e) => return Err(e),
                    }
                }
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
                                #rebind
                            )*
                            let mut q = ::ormlite::query_as(&query);
                            #(
                                #query_bindings
                            )*
                            q.fetch_one(&mut *conn)
                                .await
                                .map_err(::ormlite::Error::from)
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

    fn impl_Model__update_all_fields(&self, ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        let span = ast.span();
        let box_future = crate::util::box_future();
        let db = self.database();
        let mut placeholder = self.raw_placeholder();
        let update_clause = ast.fields()
            .map(|f| f.name())
            .filter(|f| f != attr.primary_key.as_ref().unwrap().as_str())
            .map(|f| format!("{} = {}", f, placeholder.next().unwrap()))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "UPDATE \"{}\" SET {} WHERE {} = {} RETURNING *",
            attr.table_name,
            update_clause,
            attr.primary_key.as_ref().unwrap(),
            placeholder.next().unwrap()
        );

        let id_field = syn::Ident::new(attr.primary_key.as_ref().unwrap().as_str(), span);
        let query_bindings = attr.columns.iter()
            .filter(|c| !c.is_join())
            .filter(|c| c.column_name != attr.primary_key.as_ref().unwrap().as_str())
            .map(|c| {
                let name = &c.identifier;
                quote! {
                    q = q.bind(self.#name);
                }
            });

        quote! {
            fn update_all_fields<'e, E>(self, db: E) -> #box_future<'e, ::ormlite::Result<Self>>
            where
                E: 'e +::ormlite::Executor<'e, Database = #db>,
            {
                Box::pin(async move {
                    let mut q =::ormlite::query_as::<_, Self>(#query);
                    #(#query_bindings)*
                    q.bind(self.#id_field)
                        .fetch_one(db)
                        .await
                        .map_err(::ormlite::Error::from)
                })
            }
        }
    }

    fn impl_Model__delete(&self, ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        let mut placeholder = self.raw_placeholder();
        let span = ast.span();

        let query = format!(
            "DELETE FROM \"{}\" WHERE {} = {}",
            attr.table_name,
            attr.primary_key.as_ref().unwrap(),
            placeholder.next().unwrap()
        );

        let box_future = crate::util::box_future();
        let db = self.database();
        let id_field = syn::Ident::new(attr.primary_key.as_ref().unwrap().as_str(), span);
        quote! {
            fn delete<'e, E>(self, db: E) -> #box_future<'e, ::ormlite::Result<()>>
            where
                E: 'e +::ormlite::Executor<'e, Database = #db>
            {
                Box::pin(async move {
                    let row =::ormlite::query(#query)
                        .bind(self.#id_field)
                        .execute(db)
                        .await
                        .map_err(::ormlite::Error::from)?;
                    if row.rows_affected() == 0 {
                        Err(::ormlite::Error::from(::ormlite::SqlxError::RowNotFound))
                    } else {
                        Ok(())
                    }
                })
            }
        }
    }

    fn impl_Model__fetch_one(&self, _ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        let mut placeholder = self.raw_placeholder();

        let query = format!(
            "SELECT * FROM \"{}\" WHERE {} = {}",
            attr.table_name,
            attr.primary_key.as_ref().unwrap(),
            placeholder.next().unwrap()
        );

        let db = self.database();
        let box_future = crate::util::box_future();
        quote! {
            fn fetch_one<'e, 'a, Arg, E>(id: Arg, db: E) -> #box_future<'e, ::ormlite::Result<Self>>
            where
                'a: 'e,
                Arg: 'a + Send + ::ormlite::Encode<'a, #db> + ::ormlite::types::Type<#db>,
                E: 'e +::ormlite::Executor<'e, Database = #db>
            {
                Box::pin(async move {
                    ::ormlite::query_as::<#db, Self>(#query)
                        .bind(id)
                        .fetch_one(db)
                        .await
                        .map_err(::ormlite::Error::from)
                })
            }
        }
    }


    fn impl_Model__select(&self, _ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        let table_name = &attr.table_name;
        let db = self.database();
        quote! {
            fn select<'args>() -> ::ormlite::query_builder::SelectQueryBuilder<'args, #db, Self> {
                ::ormlite::query_builder::SelectQueryBuilder::default()
                    .select(format!("\"{}\".*", #table_name))
            }
        }
    }

    fn impl_Model__builder(&self, ast: &DeriveInput, _attr: &attr::TableMetadata) -> TokenStream {
        let model = &ast.ident;
        let partial_model = quote::format_ident!("{}Builder", model.to_string());
        quote! {
                fn builder() -> #partial_model<'static> {
                    #partial_model::default()
                }
            }
    }

    fn impl_Model__update_partial(&self, ast: &DeriveInput, _attr: &attr::TableMetadata) -> TokenStream {
        let model = &ast.ident;
        let partial_model = quote::format_ident!("{}Builder", model.to_string());
        quote! {
                fn update_partial(&'slf self) -> #partial_model<'slf> {
                    let mut partial = #partial_model::default();
                    partial.updating = Some(&self);
                    partial
                }
            }
    }

    fn struct_ModelBuilder(&self, ast: &DeriveInput, _attr: &attr::TableMetadata) -> TokenStream {
        let model = &ast.ident;
        let model_builder = quote::format_ident!("{}Builder", model.to_string());
        let pub_marker = &ast.vis;

        let settable = ast.fields().map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            quote! { #name: std::option::Option<#ty> }
        });

        let methods = ast.fields().map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            if ty_is_string(&f.ty) {
                quote! {
                    pub fn #name<T: Into<String>>(mut self, #name: T) -> Self {
                        self.#name = Some(#name.into());
                        self
                    }
                }
            } else {
                quote! {
                    pub fn #name(mut self, #name: #ty) -> Self {
                        self.#name = Some(#name);
                        self
                    }
                }
            }
        });

        let build_modified_fields = ast.fields().map(|f| {
            let name = &f.ident.as_ref().unwrap();
            let name_str = name.to_string();
            quote! {
                if self.#name.is_some() {
                    ret.push(#name_str);
                }
            }
        });

        let fields_none = ast.fields().map(|f| {
            let name = &f.ident;
            quote! {
                #name: None
            }
        });

        quote! {
            #pub_marker struct #model_builder<'a> {
                #(#settable,)*
                updating: Option<&'a #model>,
            }

            impl<'a> std::default::Default for #model_builder<'a> {
                fn default() -> Self {
                    Self {
                        #(#fields_none,)*
                        updating: None,
                    }
                }
            }

            impl<'a> #model_builder<'a> {
                #(#methods)*

                fn modified_fields(&self) -> Vec<&'static str> {
                    let mut ret = Vec::new();
                    #(#build_modified_fields)*
                    ret
                }
            }
        }
    }

    fn impl_ModelBuilder__insert(&self, ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        let box_future = crate::util::box_future();
        let db = self.database();
        let placeholder = self.placeholder();
        let query = format!(
            "INSERT INTO \"{}\" ({{}}) VALUES ({{}}) RETURNING *",
            attr.table_name
        );

        let bind_parameters = generate_query_binding_code_for_partial_model(attr);

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
                    let mut q =::ormlite::query_as::<#db, Self::Model>(&query);
                    #(#bind_parameters)*
                    q.fetch_one(db)
                        .await
                        .map_err(::ormlite::Error::from)
                })
            }
        }
    }

    fn impl_ModelBuilder__update(&self, ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        let span = ast.span();
        let fields = ast.fields();

        let query = format!(
            "UPDATE \"{}\" SET {{}} WHERE {} = {{}} RETURNING *",
            attr.table_name, attr.primary_key.as_ref().unwrap(),
        );

        let db = self.database();
        let box_future = crate::util::box_future();
        let placeholder = self.placeholder();
        let bind_update = generate_query_binding_code_for_partial_model(attr);
        let id = syn::Ident::new(attr.primary_key.as_ref().unwrap().as_str(), span);
        quote! {
            fn update<'e: 'a, E>(self, db: E) -> #box_future<'a, ::ormlite::Result<Self::Model>>
            where
                E: 'e +::ormlite::Executor<'e, Database = #db>,
            {
                Box::pin(async move {
                    let mut placeholder = #placeholder;
                    let set_fields = self.modified_fields();
                    let update_id = self.updating
                        .expect("Tried to call ModelBuilder::update(), but the ModelBuilder \
                        has no reference to what model to update. You might have called \
                        something like: `<Model>::build().update(&mut db)`. A partial update \
                        looks something like \
                        `<model instance>.update_partial().update(&mut db)`.")
                        .#id
                        // NOTE: This clone is free for Copy types. .clone() fixes ormlite#13
                        .clone();
                    let query = format!(
                        #query,
                        set_fields.into_iter().map(|f| format!("{} = {}", f, placeholder.next().unwrap())).collect::<Vec<_>>().join(", "),
                        placeholder.next().unwrap()
                    );
                    let mut q =::ormlite::query_as::<#db, Self::Model>(&query);
                    #(#bind_update)*
                    q = q.bind(update_id);
                    q.fetch_one(db)
                        .await
                        .map_err(::ormlite::Error::from)
                })
            }
        }
    }

    fn impl_ModelBuilder__build(&self, ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        let span = ast.span();

        let unpack = attr.columns.iter()
            .map(|c| syn::Ident::new(c.column_name.as_str(), span))
            .map(|c| {
                let msg = format!("Tried to build a model, but the field `{c}` was not set.");
                quote! { let #c = self.#c.expect(#msg); }
            });

        let fields = attr.columns.iter()
            .map(|c| syn::Ident::new(c.column_name.as_str(), span));

        quote! {
            fn build(self) -> Self::Model {
                #( #unpack )*
                Self::Model { #( #fields, )* }
            }
        }
    }

    fn impl_ModelBuilder(&self, ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        let model = &ast.ident;
        let db = self.database();
        let partial_model = quote::format_ident!("{}Builder", model.to_string());

        let impl_ModelBuilder__insert = self.impl_ModelBuilder__insert(ast, attr);
        let impl_ModelBuilder__update = self.impl_ModelBuilder__update(ast, attr);
        let impl_ModelBuilder__build = self.impl_ModelBuilder__build(ast, attr);

        quote! {
            impl<'a> ::ormlite::model::ModelBuilder<'a, #db> for #partial_model<'a> {
                type Model = #model;
                #impl_ModelBuilder__insert
                #impl_ModelBuilder__update
                #impl_ModelBuilder__build
            }
        }
    }

    fn struct_InsertModel(&self, ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        if attr.insert_struct.is_none() {
            return quote! {};
        }
        let model_builder = quote::format_ident!("{}", attr.insert_struct.as_ref().unwrap());
        let pub_marker = &ast.vis;
        let fields = ast.fields();
        let struct_fields = fields
            .zip(attr.columns.iter())
            .filter(|(_f, col_meta)| !col_meta.has_database_default)
            .map(|(f, _)| {
                let name = &f.ident;
                let ty = &f.ty;
                quote! { pub #name: #ty }
            });

        quote! {
            #pub_marker struct #model_builder {
                #(#struct_fields,)*
            }
        }
    }

    fn impl_InsertModel(&self, ast: &DeriveInput, attr: &attr::TableMetadata) -> TokenStream {
        if attr.insert_struct.is_none() {
            return quote! {};
        }
        let box_future = crate::util::box_future();
        let model = &ast.ident;
        let db = self.database();
        let insert_model = quote::format_ident!("{}", attr.insert_struct.as_ref().unwrap());
        let fields = attr
            .columns
            .iter()
            .filter(|col_meta| !col_meta.has_database_default)
            .map(|col_meta| col_meta.column_name.clone())
            .collect::<Vec<_>>()
            .join(",");
        let mut placeholder = self.raw_placeholder();
        let placeholders = attr
            .columns
            .iter()
            .filter(|col_meta| !col_meta.has_database_default)
            .map(|_| placeholder.next().unwrap())
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING *",
            attr.table_name, fields, placeholders,
        );

        let query_bindings = attr
            .columns
            .iter()
            .filter(|col_meta| !col_meta.has_database_default)
            .map(|f| {
                let name = quote::format_ident!("{}", f.column_name);
                quote! {
                    q = q.bind(self.#name);
                }
            });

        quote! {
            impl ::ormlite::model::Insertable<#db> for #insert_model {
                type Model = #model;

                fn insert<'e, E>(self, db: E) -> #box_future<'e, ::ormlite::Result<Self::Model>>
                where
                    E: 'e +::ormlite::Executor<'e, Database = #db>,
                {
                    Box::pin(async move {
                        let mut q =::ormlite::query_as::<#db, Self::Model>(#query);
                        #(#query_bindings)*
                        q.fetch_one(db)
                            .await
                            .map_err(::ormlite::Error::from)
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ormlite_attr::*;

    #[test]
    fn test_all_bounds() {
        let mut cache = MetadataCache::new();
        let table = TableMetadata::new("user", vec![
            ColumnMetadata::new("id", "u32"),
            ColumnMetadata::new("name", "String"),
            ColumnMetadata::new("organization_id", "u32"),
            ColumnMetadata::new_join("organization", "Organization"),
        ]);
        cache.insert("User".to_string(), table.clone());
        let table = TableMetadata::new("organization", vec![
            ColumnMetadata::new("id", "u32"),
            ColumnMetadata::new("name", "String"),
            ColumnMetadata::new("is_active", "bool"),
        ]);
        cache.insert("Organization".to_string(), table.clone());

        let types_for_bound = table_primitive_types(&table, &cache);
        assert_eq!(types_for_bound, vec![
            &OtherType::new("u32"),
            &OtherType::new("String"),
            &OtherType::new("bool"),
        ]);
        let bounds = from_row_bounds(&table, &cache);
        let bounds = quote! {
            #(#bounds)*
        };
        assert_eq!(
            bounds.to_string(),
            "u32 : :: ormlite :: decode :: Decode < 'a , R :: Database > , ".to_owned() +
                "u32 : :: ormlite :: types :: Type < R :: Database > , " +

                "String : :: ormlite :: decode :: Decode < 'a , R :: Database > , " +
                "String : :: ormlite :: types :: Type < R :: Database > , " +

                "bool : :: ormlite :: decode :: Decode < 'a , R :: Database > , " +
                "bool : :: ormlite :: types :: Type < R :: Database > ,"
        );
    }
}