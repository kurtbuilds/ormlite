use crate::codegen::common::{generate_conditional_bind, insertion_binding, OrmliteCodegen};
use crate::MetadataCache;
use ormlite_attr::{Ident, TableMetadata};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_Model__update_all_fields(db: &dyn OrmliteCodegen, attr: &TableMetadata) -> TokenStream {
    let box_future = crate::util::box_fut_ts();
    let mut placeholder = db.placeholder();
    let db = db.database_ts();
    let mut query = "UPDATE \"".to_string();
    query.push_str(&attr.table_name);
    query.push_str("\" SET ");
    for c in attr.database_columns_except_pkey() {
        query.push_str(&c.column_name);
        query.push_str(" = ");
        query.push_str(&placeholder.next().unwrap());
        query.push_str(", ");
    }
    // remove the final ", "
    query.truncate(query.len() - 2);
    query.push_str(" WHERE ");
    query.push_str(&attr.pkey.column_name);
    query.push_str(" = ");
    query.push_str(&placeholder.next().unwrap());
    query.push_str(" RETURNING *");

    let id = &attr.pkey.identifier;
    let query_bindings = attr
        .database_columns_except_pkey()
        .map(|c| insertion_binding(c));

    let unwind_joins = attr.many_to_one_joins().map(|c| {
        let id = &c.identifier;
        quote! {
            let #id = &model.#id;
        }
    });

    quote! {
        fn update_all_fields<'e, E>(self, db: E) -> #box_future<'e, ::ormlite::Result<Self>>
        where
            E: 'e +::ormlite::Executor<'e, Database = #db>,
        {
            Box::pin(async move {
                let mut q =::ormlite::query_as::<_, Self>(#query);
                let model = self;
                #(#unwind_joins)*
                #(#query_bindings)*
                q.bind(model.#id)
                    .fetch_one(db)
                    .await
                    .map_err(::ormlite::Error::from)
            })
        }
    }
}

pub fn impl_ModelBuilder__update(db: &dyn OrmliteCodegen, attr: &TableMetadata) -> TokenStream {
    let box_future = crate::util::box_fut_ts();
    let placeholder = db.placeholder_ts();
    let db = db.database_ts();

    let query = format!(
        "UPDATE \"{}\" SET {{}} WHERE {} = {{}} RETURNING *",
        attr.table_name, attr.pkey.column_name,
    );

    let bind_update = attr.database_columns().map(generate_conditional_bind);
    let id = &attr.pkey.identifier;
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
