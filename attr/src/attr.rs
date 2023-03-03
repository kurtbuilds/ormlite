use structmeta::{Flag, StructMeta};
use syn::{Ident, LitStr, Path};

/// Available attributes on a struct
#[derive(StructMeta, Debug)]
pub struct ModelAttributes {
    /// The name of the table in the database. Defaults to the struct name.
    /// Example:
    /// #[ormlite(table_name = "users")]
    /// pub struct User {
    ///    pub id: i32,
    /// }
    pub table: Option<LitStr>,

    /// The struct name of an insertion struct.
    /// Example:
    /// #[ormlite(insertable = InsertUser)]
    /// pub struct User {
    ///   pub id: i32,
    /// }
    ///
    pub insertable: Option<Ident>,

    /// Set the target database. Only needed if you have multiple databases enabled.
    /// If you have a single database enabled, you don't need to set this.
    /// Even with multiple databases, you can skip this by setting a default database with the `default-<db>` feature.
    ///
    /// Currently, because methods conflict, you
    /// You can use this attribute multiple times to set multiple databases.
    /// Example:
    /// #[ormlite(database = "postgres")]
    /// #[ormlite(database = "sqlite")]
    /// pub struct User {
    ///  pub id: i32,
    /// }
    /// This will generate orm code for `User` for both the `postgres` and `sqlite` databases.
    pub database: Option<LitStr>,
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
    pub many_to_many_table: Option<Ident>,

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

    /// Skip serializing this field to/from the database. Note the field must implement `Default`.
    pub skip: Flag,

    /// Experimental: Encode this field as JSON in the database.
    /// Only applies to `derive(IntoArguments)`. For Model structs, wrap the object in `Json<..>`.
    pub experimental_encode_as_json: Flag,
}