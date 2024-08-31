use crate::ident::Ident;
use crate::ttype::{InnerType, TType};
use crate::{ColumnAttributes, SyndecodeError};
use proc_macro2::TokenStream;
use syn::Field;

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
pub struct ColumnMetadata {
    /// Name of the column in the database
    pub column_name: String,
    pub column_type: TType,
    /// Only says whether the primary key is marked (with an attribute). Use table_metadata.primary_key to definitively know the primary key.
    pub marked_primary_key: bool,
    pub has_database_default: bool,
    /// Identifier used in Rust to refer to the column
    pub identifier: Ident,

    // only for joins. Database key
    pub many_to_one_column_name: Option<String>,
    pub many_to_many_table: Option<String>,
    pub one_to_many_foreign_key: Option<ForeignKey>,

    pub skip: bool,
    pub experimental_encode_as_json: bool,
    pub rust_default: Option<String>,
    pub join: Option<Join>,
}

impl ColumnMetadata {
    pub fn is_default(&self) -> bool {
        self.rust_default.is_some() || self.has_database_default
    }
}

impl Default for ColumnMetadata {
    fn default() -> Self {
        Self {
            column_name: String::new(),
            column_type: TType::Inner(InnerType::new("String")),
            marked_primary_key: false,
            has_database_default: false,
            identifier: Ident::new("column"),
            many_to_one_column_name: None,
            many_to_many_table: None,
            one_to_many_foreign_key: None,
            skip: false,
            experimental_encode_as_json: false,
            rust_default: None,
            join: None,
        }
    }
}

impl ColumnMetadata {
    pub fn new(name: &str, ty: &str) -> Self {
        Self {
            column_name: name.to_string(),
            column_type: TType::Inner(InnerType::new(ty)),
            ..Self::default()
        }
    }

    pub fn new_join(name: &str, join_model: &str) -> Self {
        Self {
            column_name: name.to_string(),
            column_type: TType::Join(Box::new(TType::Inner(InnerType::new(join_model)))),
            ..Self::default()
        }
    }

    pub fn is_join(&self) -> bool {
        matches!(self.column_type, TType::Join(_))
    }

    pub fn is_join_many(&self) -> bool {
        let TType::Join(join) = &self.column_type else {
            return false;
        };
        let TType::Inner(o) = join.as_ref() else {
            return false;
        };
        o.ident.0 == "Vec"
    }

    pub fn is_json(&self) -> bool {
        self.column_type.is_json()
    }

    /// We expect this to only return a `Model` of some kind.
    pub fn joined_struct_name(&self) -> Option<String> {
        let TType::Join(join) = &self.column_type else {
            return None;
        };
        Some(join.inner_type_name())
    }

    pub fn joined_model(&self) -> TokenStream {
        self.column_type.qualified_inner_name()
    }
}

impl TryFrom<&Field> for ColumnMetadata {
    type Error = SyndecodeError;

    fn try_from(f: &Field) -> Result<Self, Self::Error> {
        let ident = f.ident.as_ref().expect("No ident on field");
        let syn::Type::Path(ty) = &f.ty else {
            return Err(SyndecodeError(format!("No type on field {}", ident)));
        };

        let mut result = ColumnMetadata::default();
        result.column_name = ident.to_string();
        result.identifier = Ident(ident.to_string());
        result.column_type = TType::from(&ty.path);

        for attr in f.attrs.iter().filter(|&a| a.path().is_ident("ormlite")) {
            let args: ColumnAttributes = attr.parse_args().unwrap();
            if args.primary_key.value() {
                result.marked_primary_key = true;
                result.has_database_default = true;
            }
            if args.default.value() {
                result.has_database_default = true;
            }
            if let Some(c) = args.column {
                result.column_name = c.value();
            }
            if let Some(value) = args.join_column {
                let value = value.value();
                result.many_to_one_column_name = Some(value.clone());
                result.column_name = value.clone();
                result.join = Some(Join::ManyToOne { column: value });
            }
            if let Some(path) = args.many_to_many_table {
                let value = path.to_string();
                result.many_to_many_table = Some(value);
                result.join = Some(Join::ManyToMany {});
            }
            if let Some(_path) = args.one_to_many_foreign_key {
                result.one_to_many_foreign_key = Some(ForeignKey {
                    model: "".to_string(),
                    column: "".to_string(),
                });
                result.join = Some(Join::OneToMany {});
                panic!("Join support in ormlite is in alpha state, and one_to_many_foreign_key is unfortunately not implemented yet.");
            }
            if args.skip.value() {
                result.skip = true;
            }
            if args.experimental_encode_as_json.value() {
                result.experimental_encode_as_json = true;
            }
            if let Some(default_value) = args.default_value {
                result.rust_default = Some(default_value.value());
            }
            if args.insertable_primary_key.value() {
                result.marked_primary_key = true;
            }
            if result.column_type.is_join() ^ result.join.is_some() {
                return Err(SyndecodeError(format!("Column {ident} is a Join. You must specify one of these attributes: join_column (for many to one), many_to_many_table_name, or one_to_many_foreign_key")));
            }
        }
        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub struct ForeignKey {
    pub model: String,
    pub column: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Fields, ItemStruct};

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
        let column = ColumnMetadata::try_from(field).unwrap();
        assert_eq!(column.column_name, "name");
        assert_eq!(column.column_type, TType::Inner(InnerType::new("String")));
        assert_eq!(column.marked_primary_key, false);
        assert_eq!(column.has_database_default, false);
        assert_eq!(column.rust_default, Some("\"foo\".to_string()".to_string()));
        assert_eq!(column.identifier, Ident::new("name"));
    }
}
