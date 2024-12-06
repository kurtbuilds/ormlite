use crate::{Ident, Type};
use proc_macro2::TokenStream;
use structmeta::{Flag, StructMeta};
use syn::{Attribute, Field, LitStr, Path};

#[derive(Debug, Clone)]
pub enum Join {
    ManyToOne {
        /// Name of local column on the table that maps to the fk on the other table
        column: String,
    },
    ManyToMany {},
    OneToMany {},
}

/// All the metadata we can capture about a column
#[derive(Clone, Debug)]
pub struct ColumnMeta {
    /// Name of the column in the database
    pub name: String,
    pub ty: Type,
    /// Only says whether the primary key is marked (with an attribute). Use table_metadata.primary_key to definitively know the primary key.
    pub marked_primary_key: bool,
    pub has_database_default: bool,
    /// Identifier used in Rust to refer to the column
    pub ident: Ident,

    // only for joins. Database key
    pub many_to_one_column_name: Option<String>,
    pub many_to_many_table: Option<String>,
    pub one_to_many_foreign_key: Option<ForeignKey>,

    pub skip: bool,
    pub rust_default: Option<String>,
    pub join: Option<Join>,
    pub json: bool,
}

impl ColumnMeta {
    pub fn is_default(&self) -> bool {
        self.rust_default.is_some() || self.has_database_default
    }

    pub fn from_fields<'a>(fields: impl Iterator<Item = &'a Field>) -> Vec<Self> {
        fn fun_name(f: &Field) -> ColumnMeta {
            ColumnMeta::from_field(f)
        }
        fields.map(fun_name).collect()
    }

    pub fn from_syn(ident: &syn::Ident, ty: &syn::Type) -> Self {
        let syn::Type::Path(ty) = &ty else {
            panic!("No type on field {}", ident);
        };
        Self {
            name: ident.to_string(),
            ty: Type::from(&ty.path),
            marked_primary_key: false,
            has_database_default: false,
            ident: Ident::from(ident),
            many_to_one_column_name: None,
            many_to_many_table: None,
            one_to_many_foreign_key: None,
            skip: false,
            rust_default: None,
            join: None,
            json: false,
        }
    }

    pub fn is_join(&self) -> bool {
        matches!(self.ty, Type::Join(_))
    }

    pub fn is_join_many(&self) -> bool {
        let Type::Join(join) = &self.ty else {
            return false;
        };
        let Type::Inner(o) = join.as_ref() else {
            return false;
        };
        o.ident == "Vec"
    }

    pub fn is_option(&self) -> bool {
        matches!(self.ty, Type::Option(_))
    }

    pub fn is_json(&self) -> bool {
        self.ty.is_json() || self.json
    }

    /// We expect this to only return a `Model` of some kind.
    pub fn joined_struct_name(&self) -> Option<String> {
        let Type::Join(join) = &self.ty else {
            return None;
        };
        Some(join.inner_type_name())
    }

    pub fn joined_model(&self) -> TokenStream {
        self.ty.qualified_inner_name()
    }

    pub fn from_field(f: &Field) -> Self {
        let ident = f.ident.as_ref().expect("No ident on field");
        let attrs = ColumnAttr::from_attrs(&f.attrs);
        let mut column = ColumnMeta::from_syn(ident, &f.ty);
        for attr in attrs {
            if attr.primary_key.value() {
                column.marked_primary_key = true;
                column.has_database_default = true;
            }
            if let Some(c) = attr.column {
                column.name = c.value();
            }
            if let Some(value) = attr.join_column {
                let value = value.value();
                column.many_to_one_column_name = Some(value.clone());
                column.name = value.clone();
                column.join = Some(Join::ManyToOne { column: value });
            }
            if let Some(path) = attr.many_to_many_table {
                let value = path.to_string();
                column.many_to_many_table = Some(value);
                column.join = Some(Join::ManyToMany {});
            }
            if let Some(_path) = attr.one_to_many_foreign_key {
                column.one_to_many_foreign_key = Some(ForeignKey {
                    model: "".to_string(),
                    column: "".to_string(),
                });
                column.join = Some(Join::OneToMany {});
                panic!("Join support in ormlite is in alpha state, and one_to_many_foreign_key is unfortunately not implemented yet.");
            }
            if let Some(default_value) = attr.default_value {
                column.rust_default = Some(default_value.value());
            }
            column.has_database_default |= attr.default.value();
            column.marked_primary_key |= attr.insertable_primary_key.value();
            column.skip |= attr.skip.value();
            column.json |= attr.json.value();
        }
        if column.ty.is_join() ^ column.join.is_some() {
            panic!("Column {ident} is a Join. You must specify one of these attributes: join_column (for many to one), many_to_many_table_name, or one_to_many_foreign_key");
        }
        column
    }

    #[doc(hidden)]
    pub fn mock(name: &str, ty: &str) -> Self {
        Self {
            name: name.to_string(),
            ty: Type::Inner(crate::InnerType::mock(ty)),
            marked_primary_key: false,
            has_database_default: false,
            ident: Ident::from(name),
            many_to_one_column_name: None,
            many_to_many_table: None,
            one_to_many_foreign_key: None,
            skip: false,
            rust_default: None,
            join: None,
            json: false,
        }
    }

    #[doc(hidden)]
    pub fn mock_join(name: &str, join_model: &str) -> Self {
        Self {
            name: name.to_string(),
            ty: Type::Join(Box::new(Type::Inner(crate::InnerType::mock(join_model)))),
            marked_primary_key: false,
            has_database_default: false,
            ident: Ident::from(name),
            many_to_one_column_name: None,
            many_to_many_table: None,
            one_to_many_foreign_key: None,
            skip: false,
            rust_default: None,
            join: None,
            json: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForeignKey {
    pub model: String,
    pub column: String,
}

/// Available attributes on a column (struct field)
#[derive(StructMeta)]
pub struct ColumnAttr {
    pub primary_key: Flag,
    /// Marks a primary key, but includes it in the Insert struct.
    pub insertable_primary_key: Flag,
    /// Specifies that a default exists at the database level.
    pub default: Flag,
    /// Specify a default value on the Rust side.
    pub default_value: Option<LitStr>,

    /// Note this column is not expected to exist on the model, but needs to exist in the database.
    /// Example:
    /// pub struct User {
    ///     #[ormlite(join_column = "organization_id")]
    ///     pub organization: Join<Organization>,
    /// }
    pub join_column: Option<LitStr>,

    /// Example:
    /// pub struct User {
    ///     pub org_id: i32,
    ///     #[ormlite(many_to_many_table_name = join_user_role)]
    ///     pub roles: Join<Vec<Role>>,
    /// }
    pub many_to_many_table: Option<syn::Ident>,

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

    pub json: Flag,
}

impl ColumnAttr {
    pub fn from_attrs(ast: &[Attribute]) -> Vec<Self> {
        ast.iter()
            .filter(|a| a.path().is_ident("ormlite"))
            .map(|a| a.parse_args().unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Attribute, Fields, ItemStruct};

    #[test]
    fn test_from_field() {
        let item: ItemStruct = syn::parse_str(
            r#"
struct Foo {
#[ormlite(default_value = "\"foo\".to_string()")]
pub name: String
}
"#,
        )
        .unwrap();
        let Fields::Named(fields) = item.fields else {
            panic!();
        };
        let field = fields.named.first().unwrap();
        let column = ColumnMeta::from_field(field);
        assert_eq!(column.name, "name");
        assert_eq!(column.ty, "String");
        assert!(!column.marked_primary_key);
        assert!(!column.has_database_default);
        assert_eq!(column.rust_default, Some("\"foo\".to_string()".to_string()));
        assert_eq!(column.ident, "name");
    }

    #[test]
    fn test_default() {
        let attr: Attribute = parse_quote!(#[ormlite(default_value = "serde_json::Value::Null")]);
        let args: ColumnAttr = attr.parse_args().unwrap();
        assert!(args.default_value.is_some());

        let attr: Attribute = parse_quote!(#[ormlite(default)]);
        let args: ColumnAttr = attr.parse_args().unwrap();
        assert!(args.default.value());
    }

    #[test]
    fn test_join_column() {
        let attr: Attribute = parse_quote!(#[ormlite(join_column = "org_id")]);
        let args: ColumnAttr = attr.parse_args().unwrap();
        assert!(args.join_column.is_some());
    }
}
