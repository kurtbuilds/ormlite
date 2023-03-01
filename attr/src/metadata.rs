use derive_builder::Builder;
use syn::{DeriveInput, Field, PathArguments};
use crate::{ColumnAttributes, ModelAttributes, SyndecodeError};
use crate::DeriveInputExt;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{TokenStreamExt};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Ident(pub String);

impl Ident {
    pub fn new(ident: &str) -> Self {
        Ident(ident.to_string())
    }
}

impl From<&proc_macro2::Ident> for Ident {
    fn from(ident: &proc_macro2::Ident) -> Self {
        Ident(ident.to_string())
    }
}

impl quote::ToTokens for Ident {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(proc_macro2::Ident::new(&self.0, proc_macro2::Span::call_site()))
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Segments {
    pub segments: Vec<Ident>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OtherType {
    pub path: Vec<Ident>,
    pub ident: Ident,
    pub args: Option<Box<OtherType>>,
}

impl OtherType {
    pub fn new(ident: &str) -> Self {
        Self {
            path: vec![],
            ident: Ident::new(ident),
            args: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Type {
    Option(Box<Type>),
    Vec(Box<Type>),
    /// Database primitive, includes DateTime, Jsonb, etc.
    Primitive(OtherType),
    Foreign(OtherType),
    Join(Box<Type>),
}

impl Type {
    pub fn is_primitive(&self) -> bool {
        match self {
            Type::Primitive(_) => true,
            Type::Option(ty) => ty.is_primitive(),
            _ => false,
        }

    }

    pub fn is_join(&self) -> bool {
        matches!(self, Type::Join(_))
    }

    pub fn inner_type_name(&self) -> String {
        match self {
            Type::Primitive(ty) | Type::Foreign(ty) => ty.ident.0.to_string(),
            Type::Option(ty) => ty.inner_type_name(),
            Type::Vec(ty) => ty.inner_type_name(),
            Type::Join(ty) => ty.inner_type_name(),
        }
    }

    pub fn qualified_inner_name(&self) -> TokenStream {
        match self {
            Type::Primitive(ty) | Type::Foreign(ty) => {
                let segments = ty.path.iter();
                let ident = &ty.ident;
                quote::quote! {
                    #(#segments)::* #ident
                }
            },
            Type::Option(ty) => ty.qualified_inner_name(),
            Type::Vec(ty) => ty.qualified_inner_name(),
            Type::Join(ty) => ty.qualified_inner_name(),
        }
    }
}

impl From<&syn::Path> for OtherType {
    fn from(path: &syn::Path) -> Self {
        let segment = path.segments.last().expect("path must have at least one segment");
        let args: Option<Box<OtherType>> = if let PathArguments::AngleBracketed(args) = &segment.arguments {
            let args = &args.args;
            let syn::GenericArgument::Type(ty) = args.first().unwrap() else {
                panic!("Option must have a type parameter");
            };
            let syn::Type::Path(path) = &ty else {
                panic!("Option must have a type parameter");
            };
            Some(Box::new(OtherType::from(&path.path)))
        } else {
            None
        };
        let mut path = path.segments.iter().map(|s| Ident::from(&s.ident)).collect::<Vec<_>>();
        let ident = path.pop().expect("path must have at least one segment");
        OtherType {
            path,
            args,
            ident,
        }
    }
}

impl From<OtherType> for Type {
    fn from(value: OtherType) -> Self {
        match value.ident.0.as_str() {
            "Option" => {
                let ty = value.args.unwrap();
                Type::Option(Box::new(Type::from(*ty)))
            }
            "Vec" => {
                let ty = value.args.unwrap();
                Type::Vec(Box::new(Type::from(*ty)))
            }
            "Join" => {
                let ty = value.args.unwrap();
                Type::Join(Box::new(Type::from(*ty)))
            }
            x if [
                "i8", "i16", "i32", "i64", "i128", "isize",
                "u8", "u16", "u32", "u64", "u128", "usize",
                "f32", "f64",
                "bool",
                "String",
                "str",
                "DateTime", "NaiveDate", "NaiveTime", "NaiveDateTime",
                "Decimal",
                "Uuid",
                "Json",
            ].contains(&x) => Type::Primitive(value),
            _ => Type::Foreign(value),
        }
    }
}

impl From<&syn::Path> for Type {
    fn from(path: &syn::Path) -> Self {
        let other = OtherType::from(path);
        Type::from(other)
    }
}

impl quote::ToTokens for OtherType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let args = if let Some(args) = &self.args {
            quote::quote! { <#args> }
        } else {
            quote::quote! {}
        };
        let path = &self.path;
        let ident = &self.ident;
        tokens.append_all(quote::quote! { #(#path ::)* #ident #args });
    }
}

impl quote::ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Type::Option(ty) => {
                tokens.append_all(quote::quote! { Option<#ty> });
            }
            Type::Vec(ty) => {
                tokens.append_all(quote::quote! { Vec<#ty> });
            }
            Type::Primitive(ty) | Type::Foreign(ty) => {
                ty.to_tokens(tokens);
            }
            Type::Join(ty) => {
                tokens.append_all(quote::quote! { ormlite::model::Join<#ty> });
            }
        }
    }
}

/// All the metadata we can capture about a table
#[derive(Builder, Debug, Clone)]
pub struct TableMetadata {
    pub table_name: String,
    pub struct_name: Ident,
    pub primary_key: Option<String>,
    pub columns: Vec<ColumnMetadata>,
    pub insert_struct: Option<String>,
    pub databases: Vec<String>,
}

impl Default for TableMetadata {
    fn default() -> Self {
        TableMetadata {
            table_name: String::new(),
            struct_name: Ident::new(""),
            primary_key: None,
            columns: vec![],
            insert_struct: None,
            databases: vec![],
        }
    }
}

impl TableMetadata {
    pub fn new(name: &str, columns: Vec<ColumnMetadata>) -> Self {
        TableMetadata {
            table_name: name.to_string(),
            struct_name: Ident(name.to_case(Case::Pascal)),
            primary_key: None,
            columns,
            insert_struct: None,
            databases: vec![],
        }
    }

    pub fn builder() -> TableMetadataBuilder {
        TableMetadataBuilder::default()
    }

    pub fn builder_from_struct_attributes(ast: &DeriveInput) -> Result<TableMetadataBuilder, SyndecodeError> {
        let mut builder = TableMetadata::builder();
        builder.insert_struct(None);
        builder.struct_name(Ident::from(&ast.ident));
        let mut databases = vec![];
        for attr in ast.attrs.iter().filter(|a| a.path.is_ident("ormlite")) {
            let args: ModelAttributes = attr.parse_args()
                .map_err(|e| SyndecodeError(e.to_string()))?;
            if let Some(value) = args.table {
                builder.table_name(value.value());
            }
            if let Some(value) = args.insertable {
                builder.insert_struct(Some(value.to_string()));
            }
            if let Some(value) = args.database {
                databases.push(value.value());
            }
        }
        builder.databases(databases);
        Ok(builder)
    }

    pub fn all_fields(&self, span: proc_macro2::Span) -> impl Iterator<Item=syn::Ident> + '_ {
        self.columns.iter()
            .map(move |c| syn::Ident::new(&c.column_name, span))
    }
}


impl TableMetadataBuilder {
    pub fn complete_with_struct_body(&mut self, ast: &DeriveInput) -> Result<TableMetadata, SyndecodeError> {
        let model = &ast.ident;
        let model_lowercased = model.to_string().to_case(Case::Snake);
        self.table_name.get_or_insert(model_lowercased);

        let mut cols = ast.fields()
            .map(ColumnMetadata::try_from)
            .collect::<Result<Vec<_>, _>>().unwrap();
        let mut primary_key = cols
            .iter()
            .filter(|c| c.marked_primary_key)
            .map(|c| c.column_name.clone())
            .next();
        if primary_key.is_none() {
            let candidates = sqlmo::util::pkey_column_names(&self.table_name.as_ref().unwrap());
            for c in &mut cols {
                if candidates.contains(&c.column_name) {
                    primary_key = Some(c.column_name.clone());
                    c.has_database_default = true;
                    break;
                }
            }
        }
        self.primary_key(primary_key);
        self.columns(cols);
        self.build().map_err(|e| SyndecodeError(e.to_string()))
    }
}


impl TryFrom<&DeriveInput> for TableMetadata {
    type Error = SyndecodeError;

    fn try_from(ast: &DeriveInput) -> Result<Self, Self::Error> {
        TableMetadata::builder_from_struct_attributes(ast)?
            .complete_with_struct_body(ast)
    }
}

#[derive(Clone, Debug)]
pub struct ForeignKey {
    pub model: String,
    pub column: String,
}

impl From<&syn::Path> for ForeignKey {
    fn from(_value: &syn::Path) -> Self {
        unimplemented!()
    }
}

/// All the metadata we can capture about a column
#[derive(Clone, Debug, Builder)]
pub struct ColumnMetadata {
    /// Name of the column in the database
    pub column_name: String,
    pub column_type: Type,
    /// Only says whether the primary key is marked (with an attribute). Use table_metadata.primary_key to definitively know the primary key.
    pub marked_primary_key: bool,
    pub has_database_default: bool,
    /// Identifier used in Rust to refer to the column
    pub identifier: Ident,

    // only for joins
    pub many_to_one_key: Option<Ident>,
    pub many_to_many_table: Option<String>,
    pub one_to_many_foreign_key: Option<ForeignKey>,
}

impl Default for ColumnMetadata {
    fn default() -> Self {
        Self {
            column_name: String::new(),
            column_type: Type::Primitive(OtherType::new("String")),
            marked_primary_key: false,
            has_database_default: false,
            identifier: Ident::new("column"),
            many_to_one_key: None,
            many_to_many_table: None,
            one_to_many_foreign_key: None,
        }
    }
}

impl ColumnMetadata {
    pub fn new(name: &str, ty: &str) -> Self {
        Self {
            column_name: name.to_string(),
            column_type: Type::Primitive(OtherType::new(ty)),
            ..Self::default()
        }
    }

    pub fn new_join(name: &str, join_model: &str) -> Self {
        Self {
            column_name: name.to_string(),
            column_type: Type::Join(Box::new(Type::Foreign(OtherType::new(join_model)))),
            ..Self::default()
        }
    }

    pub fn builder() -> ColumnMetadataBuilder {
        ColumnMetadataBuilder::default()
    }

    pub fn is_join(&self) -> bool {
        matches!(self.column_type, Type::Join(_))
    }

    pub fn is_join_many(&self) -> bool {
        let Type::Join(join) = &self.column_type else {
            return false;
        };
        let Type::Primitive(o) = join.as_ref() else {
            return false;
        };
        o.ident.0 == "Vec"
    }

    pub fn is_primitive(&self) -> bool {
        self.column_type.is_primitive()
    }

    /// We expect this to only return a `Model` of some kind.
    pub fn joined_struct_name(&self) -> Option<String> {
        let Type::Join(join) = &self.column_type else {
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
        let mut builder = ColumnMetadata::builder();
        let ident = f.ident.as_ref().expect("No ident on field");
        let syn::Type::Path(ty) = &f.ty else {
            return Err(SyndecodeError(format!("No type on field {}", ident)));
        };
        let ty = Type::from(&ty.path);
        let is_join = ty.is_join();
        builder
            .column_name(ident.to_string())
            .identifier(Ident::from(ident))
            .column_type(ty)
            .marked_primary_key(false)
            .has_database_default(false)
            .many_to_one_key(None)
            .many_to_many_table(None)
            .one_to_many_foreign_key(None)
        ;
        let mut has_join_directive = false;
        for attr in f.attrs.iter().filter(|a| a.path.is_ident("ormlite")) {
            let args: ColumnAttributes = attr.parse_args().unwrap();
            if args.primary_key {
                builder.marked_primary_key(true);
                builder.has_database_default(true);
            }
            if args.default {
                builder.has_database_default(true);
            }
            if let Some(column_name) = args.column {
                builder.column_name(column_name.value());
            }
            if let Some(value) = args.many_to_one_key {
                let ident = value.segments.last().unwrap().ident.clone();
                let ident = Ident::from(&ident);
                builder.many_to_one_key(Some(ident));
                has_join_directive = true;
            }
            if let Some(path) = args.many_to_many_table {
                let value = path.to_string();
                builder.many_to_many_table(Some(value));
                has_join_directive = true;
            }
            if let Some(path) = args.one_to_many_foreign_key {
                builder.one_to_many_foreign_key(Some(ForeignKey::from(&path)));
                has_join_directive = true;
            }
        }
        if is_join && !has_join_directive {
            return Err(SyndecodeError(format!("Column {ident} is a Join. You must specify one of these attributes: many_to_one_key, many_to_many_table_name, or one_to_many_foreign_key")));
        }
        builder.build().map_err(|e| SyndecodeError(e.to_string()))
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_primitive() {
        use syn::Path;
        let ty = Type::from(&syn::parse_str::<Path>("i32").unwrap());
        assert!(ty.is_primitive());

        let ty = Type::from(&syn::parse_str::<Path>("NaiveDate").unwrap());
        assert!(ty.is_primitive());

        let ty = Type::from(&syn::parse_str::<Path>("Option<NaiveDate>").unwrap());
        assert!(ty.is_primitive());

        let ty = Type::from(&syn::parse_str::<Path>("Join<User>").unwrap());
        assert!(!ty.is_primitive());

        let ty = Type::from(&syn::parse_str::<Path>("rust_decimal::Decimal").unwrap());
        assert!(ty.is_primitive());
    }

    #[test]
    fn test_other_type_to_quote() {
        use syn::Path;
        let ty = Type::from(&syn::parse_str::<Path>("rust_decimal::Decimal").unwrap());
        let Type::Primitive(ty) = &ty else {
            panic!("expected primitive");
        };
        assert_eq!(ty.ident.0, "Decimal");
        assert_eq!(ty.path.len(), 1);
        assert_eq!(ty.path[0].0.as_str(), "rust_decimal");
        let z = quote::quote!(#ty);
        assert_eq!(z.to_string(), "rust_decimal :: Decimal");
    }
}
