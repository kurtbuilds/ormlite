use crate::TableAttr;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{DeriveInput, Field};

/// Given the fields of a PartialModel struct, return the quoted code.
fn generate_query_binding_code_for_partial_model(
    fields: &Punctuated<Field, Comma>,
) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            if let Some(value) = self.#name {
                q = q.bind(value);
            }
        }
    })
}

/// Give the fields as a TokenStream of Strings (becomes string literals when used in quote!)
fn get_field_name_tokens(
    fields: &Punctuated<Field, Comma>,
) -> impl Iterator<Item = TokenStream> + '_ {
    fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string().into_token_stream())
}

fn get_field_names(fields: &Punctuated<Field, Comma>) -> impl Iterator<Item = String> + '_ {
    fields.iter().map(|f| f.ident.as_ref().unwrap().to_string())
}

fn ty_is_string(ty: &syn::Type) -> bool {
    let p = match ty {
        syn::Type::Path(p) => p,
        _ => return false,
    };
    p.path.segments.len() == 1 && p.path.segments[0].ident == "String"
}

pub trait OrmliteCodegen {
    fn impl_TableMeta(ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let model = &ast.ident;
        let table_name = &attr.table_name;
        let id = &attr.primary_key_column;
        let fields = crate::util::get_fields(ast);
        let field_names = get_field_name_tokens(fields);
        let n_fields = fields.len();

        quote! {
            impl ::ormlite::model::TableMeta for #model {
                fn table_name() -> &'static str {
                    #table_name
                }

                fn fields() -> &'static [&'static str] {
                    &[#(#field_names,)*]
                }

                fn num_fields() -> usize {
                    #n_fields
                }

                fn primary_key_column() -> &'static str {
                    #id
                }
            }
        }
    }

    fn impl_Model__insert(ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let box_future = crate::util::box_future();
        let db = Self::database();
        let fields = crate::util::get_fields(ast);
        let field_names = get_field_names(fields).collect::<Vec<_>>().join(", ");
        let mut placeholder = Self::raw_placeholder();
        let params = fields
            .iter()
            .map(|_| placeholder.next().unwrap())
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            attr.table_name, field_names, params
        );

        let query_bindings = fields.iter().map(|f| {
            let name = &f.ident;
            quote! {
                q = q.bind(self.#name);
            }
        });

        quote! {
            fn insert(self, db: &mut <#db as ::sqlx::Database>::Connection) -> #box_future<::ormlite::Result<Self>> {
                Box::pin(async move {
                    let mut q = sqlx::query_as::<#db, Self>(#query);
                    #(#query_bindings)*
                    q.fetch_one(db)
                        .await
                        .map_err(::ormlite::Error::from)
                })
            }
        }
    }

    fn impl_Model__update_all_fields(ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let box_future = crate::util::box_future();
        let db = Self::database();
        let fields = crate::util::get_fields(ast);
        let mut placeholder = Self::raw_placeholder();
        let update_clause = get_field_names(fields)
            .filter(|f| f != &attr.primary_key_column)
            .map(|f| format!("{} = {}", f, placeholder.next().unwrap()))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "UPDATE {} SET {} WHERE {} = {} RETURNING *",
            attr.table_name,
            update_clause,
            attr.primary_key_column,
            placeholder.next().unwrap()
        );

        let id_field = quote::format_ident!("{}", attr.primary_key_column);
        let query_bindings = fields
            .iter()
            .filter(|f| &f.ident.as_ref().unwrap().to_string() != &attr.primary_key_column)
            .map(|f| {
                let name = &f.ident;
                quote! {
                    q = q.bind(self.#name);
                }
            });

        quote! {
            fn update_all_fields(self, db: &mut <#db as ::sqlx::Database>::Connection) -> #box_future<::ormlite::Result<Self>> {
                Box::pin(async move {
                    let mut q = sqlx::query_as::<_, Self>(#query);
                    #(#query_bindings)*
                    q.bind(self.#id_field)
                        .fetch_one(db)
                        .await
                        .map_err(::ormlite::Error::from)
                })
            }
        }
    }

    fn impl_Model__delete(_ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let mut placeholder = Self::raw_placeholder();

        let query = format!(
            "DELETE FROM {} WHERE {} = {}",
            attr.table_name,
            attr.primary_key_column,
            placeholder.next().unwrap()
        );

        let box_future = crate::util::box_future();
        let db = Self::database();
        let id_field = quote::format_ident!("{}", attr.primary_key_column);
        quote! {
           fn delete(self, db: &mut <#db as ::sqlx::Database>::Connection) -> #box_future<::ormlite::Result<()>> {
                Box::pin(async move {
                    let row = ::sqlx::query(#query)
                        .bind(self.#id_field)
                        .execute(db)
                        .await
                        .map_err(::ormlite::Error::from)?;
                    if row.rows_affected() == 0 {
                        Err(::ormlite::Error::from(::sqlx::Error::RowNotFound))
                    } else {
                        Ok(())
                    }
                })
            }
        }
    }

    fn impl_Model__get_one(_ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let mut placeholder = Self::raw_placeholder();

        let query = format!(
            "SELECT * FROM {} WHERE {} = {}",
            attr.table_name,
            attr.primary_key_column,
            placeholder.next().unwrap()
        );

        let db = Self::database();
        let box_future = crate::util::box_future();
        quote! {
            fn get_one<'db, 'arg: 'db, T>(
                id: T,
                db: &'db mut <#db as ::sqlx::Database>::Connection,
            ) ->
                #box_future<'db, ::ormlite::Result<Self>>
                where
                    T: 'arg + Send + for<'r> ::sqlx::Encode<'r, #db> + ::sqlx::Type<#db>,
            {
                Box::pin(async move {
                    ::sqlx::query_as::<#db, Self>(#query)
                        .bind(id)
                        .fetch_one(db)
                        .await
                        .map_err(::ormlite::Error::from)
                })
            }
        }
    }

    fn impl_Model(ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let db = Self::database();
        let model = &ast.ident;

        let impl_TableMeta = Self::impl_TableMeta(ast, attr);
        let impl_Model__insert = Self::impl_Model__insert(ast, attr);
        let impl_Model__update_all_fields = Self::impl_Model__update_all_fields(ast, attr);
        let impl_Model__delete = Self::impl_Model__delete(ast, attr);
        let impl_Model__get_one = Self::impl_Model__get_one(ast, attr);

        quote! {
            // impl #model : ::ormlite::model::Model<#db> {}
            impl ::ormlite::model::Model<#db> for #model {
                #impl_Model__insert
                #impl_Model__update_all_fields
                #impl_Model__delete
                #impl_Model__get_one

               fn query(query: &str) -> ::sqlx::query::QueryAs<#db, Self, <#db as ::sqlx::database::HasArguments>::Arguments> {
                    ::sqlx::query_as::<_, Self>(query)
                }
            }
            #impl_TableMeta
        }
    }

    fn impl_BuildsQueryBuilder(ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let table_name = &attr.table_name;
        let model = &ast.ident;
        let db = Self::database();
        quote! {
            impl ::ormlite::model::BuildsQueryBuilder<#db, ::std::boxed::Box<dyn Iterator<Item=String>>> for #model {
                fn select<'args>() -> ::ormlite::SelectQueryBuilder<'args, #db, Self, Box<dyn Iterator<Item=String>>> {
                    ::ormlite::SelectQueryBuilder::default()
                        .column(&format!("{}.*", #table_name))
                }
            }
        }
    }

    fn impl_BuildsPartialModel(ast: &DeriveInput, _attr: &TableAttr) -> TokenStream {
        let model = &ast.ident;
        let partial_model = quote::format_ident!("Partial{}", model.to_string());
        quote! {
            impl<'a> ormlite::model::BuildsPartialModel<'a, #partial_model<'a>> for #model {
                fn build() -> #partial_model<'a> {
                    #partial_model::default()
                }

                fn update_partial(&'a self) -> #partial_model<'a> {
                    let mut partial = #partial_model::default();
                    partial.updating = Some(&self);
                    partial
                }
            }
        }
    }

    fn struct_PartialModel(ast: &DeriveInput, _attr: &TableAttr) -> TokenStream {
        let base_ident = &ast.ident;
        let partial_ident = syn::Ident::new(
            &("Partial".to_string() + ast.ident.to_string().as_str()),
            proc_macro2::Span::call_site(),
        );
        let pub_marker = &ast.vis;

        let fields = crate::util::get_fields(ast);

        let settable = fields.iter().map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            quote! { #name: std::option::Option<#ty> }
        });

        let methods = fields.iter().map(|f| {
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

        let build_modified_fields = fields.iter().map(|f| {
            let name = &f.ident.as_ref().unwrap();
            let name_str = name.to_string();
            quote! {
                if self.#name.is_some() {
                    ret.push(#name_str);
                }
            }
        });

        let fields_none = fields.iter().map(|f| {
            let name = &f.ident;
            quote! {
                #name: None
            }
        });

        quote! {
            #pub_marker struct #partial_ident<'a> {
                #(#settable,)*
                updating: Option<&'a #base_ident>,
            }

            impl<'a> std::default::Default for #partial_ident<'a> {
                fn default() -> Self {
                    Self {
                        #(#fields_none,)*
                        updating: None,
                    }
                }
            }

            impl<'a> #partial_ident<'a> {
                #(#methods)*

                fn modified_fields(&self) -> Vec<&'static str> {
                    let mut ret = Vec::new();
                    #(#build_modified_fields)*
                    ret
                }
            }
        }
    }

    fn impl_PartialModel__insert(ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let box_future = crate::util::box_future();
        let db = Self::database();
        let placeholder = Self::placeholder();
        let query = format!(
            "INSERT INTO {} ({{}}) VALUES ({{}}) RETURNING *",
            attr.table_name
        );

        let fields = crate::util::get_fields(ast);
        let bind_parameters = generate_query_binding_code_for_partial_model(fields);

        quote! {
            fn insert<'db: 'a>(self, db: &'db mut <#db as ::sqlx::Database>::Connection) -> #box_future<'a, ::ormlite::Result<Self::Model>> {
                Box::pin(async move {
                    let mut placeholder = #placeholder;
                    let set_fields = self.modified_fields();
                    let query = format!(
                        #query,
                        set_fields.join(", "),
                        set_fields.iter().map(|_| placeholder.next().unwrap()).collect::<Vec<_>>().join(", "),
                    );
                    let mut q = ::sqlx::query_as::<#db, Self::Model>(&query);
                    #(#bind_parameters)*
                    q.fetch_one(db)
                        .await
                        .map_err(::ormlite::Error::from)
                })
            }
        }
    }

    fn impl_PartialModel__update(ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let fields = crate::util::get_fields(ast);

        let query = format!(
            "UPDATE {} SET {{}} WHERE {} = {{}} RETURNING *",
            attr.table_name, attr.primary_key_column,
        );

        let db = Self::database();
        let box_future = crate::util::box_future();
        let placeholder = Self::placeholder();
        let bind_update = generate_query_binding_code_for_partial_model(fields);
        let id = quote::format_ident!("{}", attr.primary_key_column);
        quote! {
            fn update<'db: 'a>(self, db: &'db mut <#db as ::sqlx::Database>::Connection) -> #box_future<'a, ::ormlite::Result<Self::Model>> {
                Box::pin(async move {
                    let mut placeholder = #placeholder;
                    let set_fields = self.modified_fields();
                    let query = format!(
                        #query,
                        set_fields.into_iter().map(|f| format!("{} = {}", f, placeholder.next().unwrap())).collect::<Vec<_>>().join(", "),
                        self.updating
                            .expect("Tried to call PartialModel::update(), but the PartialModel \
                            has no reference to what model to update. You might have called \
                            something like: `<Model>::build().update(&mut db)`. A partial update \
                            looks something like \
                            `<model instance>.update_partial().update(&mut db)`.")
                            .#id
                    );
                    let mut q = ::sqlx::query_as::<#db, Self::Model>(&query);
                    #(#bind_update)*
                    q.fetch_one(db)
                        .await
                        .map_err(::ormlite::Error::from)
                })
            }
        }
    }
    fn impl_PartialModel(ast: &DeriveInput, attr: &TableAttr) -> TokenStream {
        let model = &ast.ident;
        let db = Self::database();
        let partial_model = quote::format_ident!("Partial{}", model.to_string());

        let impl_PartialModel__insert = Self::impl_PartialModel__insert(ast, attr);
        let impl_PartialModel__update = Self::impl_PartialModel__update(ast, attr);

        quote! {
            impl<'a> ::ormlite::model::PartialModel<'a, #db> for #partial_model<'a> {
                type Model = #model;
                #impl_PartialModel__insert
                #impl_PartialModel__update
            }
        }
    }

    fn database() -> TokenStream;
    fn placeholder() -> TokenStream;
    fn raw_placeholder() -> Box<dyn Iterator<Item = String>>;
}
