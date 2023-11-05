use proc_macro2::{TokenStream};
use syn::Meta;

#[derive(Debug)]
pub struct Derives {
    pub is_model: bool,
    pub is_type: bool,
    pub is_manual_type: bool,
}

#[derive(Debug)]
pub struct Attributes2 {
    pub derives: Derives,
    pub repr: Option<String>,
}

impl Attributes2 {
    pub fn has_derive(&self, trait_name: &str) -> bool {
        match trait_name {
            "Model" => self.derives.is_model,
            "Type" => self.derives.is_type,
            "ManualType" => self.derives.is_manual_type,
            _ => false,
        }
    }
}

fn decode_traits_from_derive_tokens(derives: &mut Derives, tokens: &TokenStream) {
    use proc_macro2::TokenTree;

    let mut iter = tokens.clone().into_iter().peekable();

    while let Some(tok) = iter.next() {
        match tok {
            TokenTree::Ident(i) => {
                let p = iter.peek();
                if p.is_none() || matches!(p.unwrap(), TokenTree::Punct(p) if p.as_char() == ',') {
                    match i.to_string().as_str() {
                        "Model" => derives.is_model = true,
                        "Type" => derives.is_type = true,
                        "ManualType" => derives.is_manual_type = true,
                        _ => {}
                    }
                    _ = iter.next();
                    continue;
                } else if matches!(p.unwrap(), TokenTree::Punct(p) if p.as_char() == ':') {
                    let ns = i.to_string();
                    if !(ns == "sqlx" || ns == "ormlite") {
                        iter.find(|tok| matches!(tok, TokenTree::Punct(p) if p.as_char() == ','));
                        continue;
                    }
                    _ = iter.next(); // consume the colon
                    _ = iter.next(); // consume another colon
                    continue;
                } else {
                    panic!("Unexpected token in derive attribute: {:?}", p);
                }
            }
            _ => panic!("Unexpected token in derive attribute: {:?}", tok),
        }
    }
}

impl From<&[syn::Attribute]> for Attributes2 {
    fn from(attrs: &[syn::Attribute]) -> Self {
        let mut repr = None;
        let mut derives = Derives {
            is_model: false,
            is_type: false,
            is_manual_type: false,
        };
        for attr in attrs.iter() {
            match &attr.meta {
                Meta::List(l) => {
                    let Some(ident) = l.path.get_ident() else { continue; };
                    if ident == "derive" {
                        decode_traits_from_derive_tokens(&mut derives, &l.tokens);
                    } else if ident == "repr" {
                        let ident = l.tokens.clone().into_iter().next().expect("Encountered a repr attribute without an argument.");
                        repr = Some(ident.to_string());
                    }
                }
                _ => {}
            }
        }
        Attributes2 { derives, repr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let attrs = Attributes2::from(item.attrs.as_slice());
        assert!(attrs.has_derive("Type"));
        assert_eq!(attrs.repr, Some("u8".to_string()));
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
        let attr = Attributes2::from(item.attrs.as_slice());
        assert_eq!(attr.repr, Some("u8".to_string()));
        assert_eq!(attr.derives.is_model, true);
        assert_eq!(attr.derives.is_type, true);
        assert_eq!(attr.derives.is_manual_type, false);

    }
}
