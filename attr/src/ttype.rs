use crate::Ident;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::PathArguments;

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Type {
    Option(Box<Type>),
    Vec(Box<Type>),
    /// Database primitive, includes DateTime, Jsonb, etc.
    Inner(InnerType),
    Join(Box<Type>),
}

impl Type {
    pub fn joined_type(&self) -> Option<&Type> {
        match &self {
            Type::Join(ty) => Some(ty.as_ref()),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Type::Inner(ty) => ty.ident == "String",
            _ => false,
        }
    }

    pub fn is_json(&self) -> bool {
        match self {
            Type::Inner(ty) => ty.ident == "Json",
            Type::Option(ty) => ty.is_json(),
            _ => false,
        }
    }

    pub fn is_join(&self) -> bool {
        matches!(self, Type::Join(_))
    }

    pub fn is_option(&self) -> bool {
        matches!(self, Type::Option(_))
    }

    pub fn inner_type_name(&self) -> String {
        match self {
            Type::Inner(ty) => ty.ident.to_string(),
            Type::Option(ty) => ty.inner_type_name(),
            Type::Vec(ty) => ty.inner_type_name(),
            Type::Join(ty) => ty.inner_type_name(),
        }
    }

    pub fn inner_type_mut(&mut self) -> &mut InnerType {
        match self {
            Type::Inner(ty) => ty,
            Type::Option(ty) => ty.inner_type_mut(),
            Type::Vec(ty) => ty.inner_type_mut(),
            Type::Join(ty) => ty.inner_type_mut(),
        }
    }

    pub fn inner_type(&self) -> &InnerType {
        match self {
            Type::Inner(ty) => ty,
            Type::Option(ty) => ty.inner_type(),
            Type::Vec(ty) => ty.inner_type(),
            Type::Join(ty) => ty.inner_type(),
        }
    }

    pub fn qualified_inner_name(&self) -> TokenStream {
        match self {
            Type::Inner(ty) => {
                let segments = ty.path.iter();
                let ident = &ty.ident;
                quote::quote! {
                    #(#segments)::* #ident
                }
            }
            Type::Option(ty) => ty.qualified_inner_name(),
            Type::Vec(ty) => ty.qualified_inner_name(),
            Type::Join(ty) => ty.qualified_inner_name(),
        }
    }
}

impl From<InnerType> for Type {
    fn from(value: InnerType) -> Self {
        match value.ident.to_string().as_str() {
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
            _ => Type::Inner(value),
        }
    }
}

impl From<&syn::Path> for Type {
    fn from(path: &syn::Path) -> Self {
        let other = InnerType::from(path);
        Type::from(other)
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
            Type::Inner(ty) => {
                ty.to_tokens(tokens);
            }
            Type::Join(ty) => {
                tokens.append_all(quote::quote! { ormlite::model::Join<#ty> });
            }
        }
    }
}

impl PartialEq<&str> for Type {
    fn eq(&self, other: &&str) -> bool {
        let Type::Inner(t) = self else {
            return false;
        };
        t.ident == other
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InnerType {
    pub path: Vec<Ident>,
    pub ident: Ident,
    pub args: Option<Box<InnerType>>,
}

impl InnerType {
    #[doc(hidden)]
    pub fn mock(ident: &str) -> Self {
        Self {
            path: vec![],
            ident: Ident::from(ident),
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
                panic!("expected type syntax tree inside angle brackets");
            };
            let syn::Type::Path(path) = &ty else {
                panic!("expected path syntax tree inside angle brackets");
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
        let ty = Type::from(&syn::parse_str::<Path>("i32").unwrap());
        assert!(!ty.is_json());

        let ty = Type::from(&syn::parse_str::<Path>("Json<User>").unwrap());
        assert!(ty.is_json());
    }

    #[test]
    fn test_other_type_to_quote() {
        use syn::Path;
        let ty = Type::from(&syn::parse_str::<Path>("rust_decimal::Decimal").unwrap());
        let Type::Inner(ty) = &ty else {
            panic!("expected primitive");
        };
        assert_eq!(ty.ident, "Decimal");
        assert_eq!(ty.path.len(), 1);
        assert_eq!(ty.path[0], "rust_decimal");
        let z = quote::quote!(#ty);
        assert_eq!(z.to_string(), "rust_decimal :: Decimal");
    }
}
