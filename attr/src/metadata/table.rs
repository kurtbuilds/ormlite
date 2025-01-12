use crate::metadata::column::ColumnMeta;
use crate::DeriveInputExt;
use crate::Ident;
use convert_case::{Case, Casing};
use structmeta::StructMeta;
use syn::{Attribute, DeriveInput, LitStr};

/// Metadata used for IntoArguments, TableMeta, and (subset of) Model
/// This structs are constructed from the *Attribute structs in crate::attr.
#[derive(Debug, Clone)]
pub struct TableMeta {
    pub name: String,
    pub ident: Ident,
    pub columns: Vec<ColumnMeta>,
    pub databases: Vec<String>,

    /// If you're using this, consider whether you should be using a ModelMetadata and its pkey,
    /// which is not optional, instead.
    pub pkey: Option<String>,
}

impl TableMeta {
    pub fn new(ast: &DeriveInput, attrs: &[TableAttr]) -> Self {
        let ident = &ast.ident;
        let name = if let Some(value) = attrs.iter().find_map(|a| a.table.as_ref()) {
            value.value()
        } else {
            ident.to_string().to_case(Case::Snake)
        };
        let mut columns = ColumnMeta::from_fields(ast.fields());
        let mut pkey = columns
            .iter()
            .find(|&c| c.marked_primary_key)
            .map(|c| c.clone())
            .map(|c| c.name.clone());
        if pkey.is_none() {
            let candidates = sqlmo::util::pkey_column_names(&name);
            if let Some(c) = columns.iter_mut().find(|c| candidates.iter().any(|n| c.ident == n)) {
                c.has_database_default = true;
                pkey = Some(c.name.clone());
            }
        }
        let databases = attrs.iter().flat_map(|d| &d.database).map(|d| d.value()).collect();
        Self {
            name,
            ident: Ident::from(ident),
            columns,
            databases,
            pkey,
        }
    }

    pub fn from_derive(ast: &DeriveInput) -> Self {
        let attr = TableAttr::from_attrs(&ast.attrs);
        Self::new(ast, &attr)
    }

    pub fn all_fields(&self) -> impl Iterator<Item = &Ident> + '_ {
        self.columns.iter().map(|c| &c.ident)
    }

    pub fn database_columns(&self) -> impl Iterator<Item = &ColumnMeta> + '_ {
        self.columns
            .iter()
            .filter(|&c| !c.skip)
            .filter(|&c| !c.is_join() || c.is_join_one())
    }

    pub fn many_to_one_joins(&self) -> impl Iterator<Item = &ColumnMeta> + '_ {
        self.columns.iter().filter(|&c| c.is_join_one())
    }

    #[allow(dead_code)]
    pub(crate) fn mock(name: &str, columns: Vec<ColumnMeta>) -> Self {
        TableMeta {
            name: name.to_string(),
            ident: Ident::from(name.to_case(Case::Pascal)),
            pkey: None,
            columns,
            databases: vec![],
        }
    }
}

/// Available attributes on a struct
#[derive(StructMeta)]
pub struct TableAttr {
    /// The name of the table in the database. Defaults to the struct name.
    /// Example:
    /// #[ormlite(table = "users")]
    /// pub struct User {
    ///    pub id: i32,
    /// }
    pub table: Option<LitStr>,

    /// Deprecated name for insert
    /// Used as `#[ormlite(insertable = InsertUser)]`
    pub insertable: Option<syn::Ident>,

    /// The struct name of an insertion struct.
    /// Example:
    /// #[ormlite(insert = "InsertUser")]
    /// pub struct User {
    ///   pub id: i32,
    /// }
    ///
    pub insert: Option<LitStr>,

    /// Add extra derives to the insertion structs.
    /// Example:
    /// #[ormlite(insert = "InsertUser", extra_derives(Serialize, Deserialize))]
    /// pub struct User {
    ///   pub id: i32,
    /// }
    ///
    pub extra_derives: Option<Vec<syn::Ident>>,

    /// Only used for derive(Insert)
    /// Example:
    /// #[ormlite(returns = "User")]
    /// pub struct InsertUser {}
    pub returns: Option<LitStr>,

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

impl TableAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Vec<Self> {
        attrs
            .iter()
            .filter(|&a| a.path().is_ident("ormlite"))
            .map(|a| a.parse_args().unwrap())
            .collect()
    }
}
