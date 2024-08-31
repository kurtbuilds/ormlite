use crate::ident::Ident;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::PathArguments;

/// Token type. A rust AST token, representing a type.
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum TType {
    Option(Box<TType>),
    Vec(Box<TType>),
    /// Database primitive, includes DateTime, Jsonb, etc.
    Inner(InnerType),
    Join(Box<TType>),
}

impl TType {
    pub fn joined_type(&self) -> Option<&TType> {
        match &self {
            TType::Join(ty) => Some(ty.as_ref()),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            TType::Inner(ty) => ty.ident.0 == "String",
            _ => false,
        }
    }

    pub fn is_json(&self) -> bool {
        match self {
            TType::Inner(ty) => ty.ident.0 == "Json",
            TType::Option(ty) => ty.is_json(),
            _ => false,
        }
    }

    pub fn is_join(&self) -> bool {
        matches!(self, TType::Join(_))
    }

    pub fn is_option(&self) -> bool {
        matches!(self, TType::Option(_))
    }

    pub fn inner_type_name(&self) -> String {
        match self {
            TType::Inner(ty) => ty.ident.0.to_string(),
            TType::Option(ty) => ty.inner_type_name(),
            TType::Vec(ty) => ty.inner_type_name(),
            TType::Join(ty) => ty.inner_type_name(),
        }
    }

    pub fn inner_type_mut(&mut self) -> &mut InnerType {
        match self {
            TType::Inner(ty) => ty,
            TType::Option(ty) => ty.inner_type_mut(),
            TType::Vec(ty) => ty.inner_type_mut(),
            TType::Join(ty) => ty.inner_type_mut(),
        }
    }

    pub fn inner_type(&self) -> &InnerType {
        match self {
            TType::Inner(ty) => ty,
            TType::Option(ty) => ty.inner_type(),
            TType::Vec(ty) => ty.inner_type(),
            TType::Join(ty) => ty.inner_type(),
        }
    }

    pub fn qualified_inner_name(&self) -> TokenStream {
        match self {
            TType::Inner(ty) => {
                let segments = ty.path.iter();
                let ident = &ty.ident;
                quote::quote! {
                    #(#segments)::* #ident
                }
            }
            TType::Option(ty) => ty.qualified_inner_name(),
            TType::Vec(ty) => ty.qualified_inner_name(),
            TType::Join(ty) => ty.qualified_inner_name(),
        }
    }
}

impl From<InnerType> for TType {
    fn from(value: InnerType) -> Self {
        match value.ident.0.as_str() {
            "Option" => {
                let ty = value.args.unwrap();
                TType::Option(Box::new(TType::from(*ty)))
            }
            "Vec" => {
                let ty = value.args.unwrap();
                TType::Vec(Box::new(TType::from(*ty)))
            }
            "Join" => {
                let ty = value.args.unwrap();
                TType::Join(Box::new(TType::from(*ty)))
            }
            _ => TType::Inner(value),
        }
    }
}

impl From<&syn::Path> for TType {
    fn from(path: &syn::Path) -> Self {
        let other = InnerType::from(path);
        TType::from(other)
    }
}

impl quote::ToTokens for TType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TType::Option(ty) => {
                tokens.append_all(quote::quote! { Option<#ty> });
            }
            TType::Vec(ty) => {
                tokens.append_all(quote::quote! { Vec<#ty> });
            }
            TType::Inner(ty) => {
                ty.to_tokens(tokens);
            }
            TType::Join(ty) => {
                tokens.append_all(quote::quote! { ormlite::model::Join<#ty> });
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InnerType {
    pub path: Vec<Ident>,
    pub ident: Ident,
    pub args: Option<Box<InnerType>>,
}

impl InnerType {
    pub fn new(ident: &str) -> Self {
        Self {
            path: vec![],
            ident: Ident::new(ident),
            args: None,
        }
    }
}

impl From<&syn::Path> for InnerType {
    fn from(path: &syn::Path) -> Self {
        let segment = path.segments.last().expect("path must have at least one segment");
        let args: Option<Box<InnerType>> = if let PathArguments::AngleBracketed(args) = &segment.arguments {
            let args = &args.args;
            let syn::GenericArgument::Type(ty) = args.first().unwrap() else {
                panic!("Option must have a type parameter");
            };
            let syn::Type::Path(path) = &ty else {
                panic!("Option must have a type parameter");
            };
            Some(Box::new(InnerType::from(&path.path)))
        } else {
            None
        };
        let mut path = path.segments.iter().map(|s| Ident::from(&s.ident)).collect::<Vec<_>>();
        let ident = path.pop().expect("path must have at least one segment");
        InnerType { path, args, ident }
    }
}

impl quote::ToTokens for InnerType {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive() {
        use syn::Path;
        let ty = TType::from(&syn::parse_str::<Path>("i32").unwrap());
        assert!(!ty.is_json());

        let ty = TType::from(&syn::parse_str::<Path>("Json<User>").unwrap());
        assert!(ty.is_json());
    }

    #[test]
    fn test_other_type_to_quote() {
        use syn::Path;
        let ty = TType::from(&syn::parse_str::<Path>("rust_decimal::Decimal").unwrap());
        let TType::Inner(ty) = &ty else {
            panic!("expected primitive");
        };
        assert_eq!(ty.ident.0, "Decimal");
        assert_eq!(ty.path.len(), 1);
        assert_eq!(ty.path[0].0.as_str(), "rust_decimal");
        let z = quote::quote!(#ty);
        assert_eq!(z.to_string(), "rust_decimal :: Decimal");
    }
}
