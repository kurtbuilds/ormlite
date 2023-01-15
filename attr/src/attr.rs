use structmeta::StructMeta;
use syn::{Ident, LitStr, Path};

/// Available attributes on a struct
#[derive(StructMeta, Debug)]
pub struct ModelAttributes {
    pub table: Option<LitStr>,
    pub Insertable: Option<Ident>,
}

/// Available attributes on a column (struct field)
#[derive(StructMeta, Debug)]
pub struct ColumnAttributes {
    pub primary_key: bool,
    pub default: bool,

    /// Example:
    /// pub struct User {
    ///     pub org_id: i32,
    ///     #[ormlite(many_to_one_key = org_id)]
    ///     pub organization: Join<Organization>,
    /// }
    pub many_to_one_key: Option<Path>,

    /// Example:
    /// pub struct User {
    ///     pub org_id: i32,
    ///     #[ormlite(many_to_many_table_name = join_user_role)]
    ///     pub roles: Join<Vec<Role>>,
    /// }
    pub many_to_many_table: Option<Path>,

    /// Example:
    /// pub struct User {
    ///     pub id: i32,
    ///     #[ormlite(one_to_many_foreign_key = Post::author_id)]
    ///     pub posts: Join<Vec<Post>>,
    /// }
    ///
    /// pub struct Post {
    ///     pub id: i32,
    ///     pub author_id: i32,
    /// }
    pub one_to_many_foreign_key: Option<Path>,

    /// The name of the column in the database. Defaults to the field name.
    pub column: Option<LitStr>,

    // /// Skip serializing this field to/from the database.
    // pub skip: bool,
}