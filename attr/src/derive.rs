use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Meta, Path, Token};

use crate::cfg_attr::CfgAttr;

#[derive(Debug)]
pub struct DeriveTrait {
    /// The derived trait
    pub name: String,
    /// The path to the derived trait
    pub path: Option<String>,
}

impl DeriveTrait {
    pub fn has_derive(&self, pkg: &str, name: &str) -> bool {
        if self.name != name {
            return false;
        }
        match &self.path {
            None => true,
            Some(path) => path == pkg,
        }
    }

    pub fn has_any_derive(&self, pkg: &[&str], name: &str) -> bool {
        if self.name != name {
            return false;
        }
        match &self.path {
            None => true,
            Some(path) => pkg.contains(&path.as_str()),
        }
    }
}

impl From<Path> for DeriveTrait {
    fn from(value: Path) -> Self {
        let name = value.segments.last().as_ref().unwrap().ident.to_string();
        let mut path = None;
        if value.segments.len() > 1 {
            path = value.segments.first().map(|s| s.ident.to_string());
        }
        DeriveTrait { name, path }
    }
}

/// Existing libraries like `structmeta` and `darling` do not parse derives, as they are
/// built assuming the data comes from proc-macro, whereas in ormlite we do both proc-macro
/// as well as static codebase analysis
#[derive(Debug, Default)]
pub struct DeriveParser {
    derives: Vec<DeriveTrait>,
}

impl DeriveParser {
    pub fn has_derive(&self, pkg: &str, name: &str) -> bool {
        self.derives.iter().any(|d| d.has_derive(pkg, name))
    }

    pub fn has_any_derive(&self, pkg: &[&str], name: &str) -> bool {
        self.derives.iter().any(|d| d.has_any_derive(pkg, name))
    }

    pub(crate) fn update(&mut self, other: Derive) {
        for path in other.inner {
            self.derives.push(path.into());
        }
    }
}

impl DeriveParser {
    const ATTRIBUTE: &'static str = "derive";

    pub fn from_attributes(attrs: &[syn::Attribute]) -> Self {
        let mut result = Self::default();
        for attr in attrs {
            let Some(ident) = attr.path().get_ident() else {
                continue;
            };
            if ident == Self::ATTRIBUTE {
                result.update(attr.parse_args().unwrap());
            } else if ident == "cfg_attr" {
                let cfg: CfgAttr = attr.parse_args().unwrap();
                for attr in cfg.attrs {
                    let Some(ident) = attr.path().get_ident() else {
                        continue;
                    };
                    if ident == Self::ATTRIBUTE {
                        let Meta::List(attrs) = attr else {
                            panic!("Expected a list of attributes")
                        };
                        result.update(attrs.parse_args().unwrap());
                    }
                }
            }
        }
        result
    }
}

/// Parses `#[derive(...)]`
pub(crate) struct Derive {
    inner: Punctuated<Path, Token![,]>,
}

impl Parse for Derive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Derive {
            inner: input.parse_terminated(Path::parse_mod_style, Token![,])?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repr::Repr;

    #[test]
    fn test_repr() {
        let q = quote::quote! {
            #[derive(sqlx::Type)]
            #[repr(u8)]
            pub enum Foo {
                Bar,
                Baz,
            }
        };
        let item = syn::parse2::<syn::ItemEnum>(q).unwrap();
        let derive = DeriveParser::from_attributes(&item.attrs);
        let repr = Repr::from_attributes(&item.attrs).unwrap();
        dbg!(&derive);
        assert!(derive.has_any_derive(&["ormlite", "sqlx"], "Type"));
        assert_eq!(repr, "u8");
    }

    /// The attributes on this are sort of nonsense, but we want to test the dynamic attribute parsing
    /// in ormlite_attr::Attribute
    #[test]
    fn test_attributes() {
        // the doc string is the regression test
        let code = r#"/// Json-serializable representation of query results
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, ormlite::Model)]
#[repr(u8)]
#[ormlite(table = "result")]
#[deprecated]
pub struct QuerySet {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}"#;
        let file: syn::File = syn::parse_str(code).unwrap();
        let syn::Item::Struct(item) = file.items.first().unwrap() else {
            panic!("expected struct");
        };
        let attr = DeriveParser::from_attributes(&item.attrs);
        let repr = Repr::from_attributes(&item.attrs).unwrap();
        assert_eq!(repr, "u8");
        assert!(attr.has_derive("ormlite", "Model"));
        assert!(attr.has_any_derive(&["ormlite", "sqlx"], "Type"));
        assert!(!attr.has_derive("ormlite", "ManualType"));
    }

    #[test]
    fn test_cfg_attr() {
        // the doc string is the regression test
        let code = r#"
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(tsify::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(
        sqlx::Type,
        strum::IntoStaticStr,
        strum::EnumString,
    ),
    strum(serialize_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum Privacy {
    Private,
    Team,
    Public,
}
"#;
        let file: syn::File = syn::parse_str(code).unwrap();
        let syn::Item::Enum(item) = file.items.first().unwrap() else {
            panic!()
        };
        let attr = DeriveParser::from_attributes(&item.attrs);
        assert!(attr.has_any_derive(&["ormlite", "sqlx"], "Type"));
    }

    #[test]
    fn test_cfg_attr2() {
        let code = r#"
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(tsify::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(ormlite::types::ManualType, strum::IntoStaticStr, strum::EnumString),
    strum(serialize_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum Privacy {
    Private,
    Team,
    Public,
}
"#;
        let file: syn::File = syn::parse_str(code).unwrap();
        let syn::Item::Enum(item) = file.items.first().unwrap() else {
            panic!()
        };
        let attr = DeriveParser::from_attributes(&item.attrs);
        assert_eq!(attr.has_derive("ormlite", "ManualType"), true);
    }
}
