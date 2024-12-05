use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

#[proc_macro_derive(Enum)]
pub fn derive_ormlite_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = input.ident;

    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("#[derive(OrmliteEnum)] is only supported on enums"),
    };

    // Collect variant names and strings into vectors
    let variant_names: Vec<_> = variants.iter().map(|v| &v.ident).collect();
    let variant_strings: Vec<_> = variant_names.iter().map(|v| v.to_string().to_lowercase()).collect();

    let gen = quote! {
        use std::str::FromStr;
        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(Self::#variant_names => write!(f, "{}", #variant_strings)),*
                }
            }
        }

        impl std::str::FromStr for #enum_name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#variant_strings => Ok(Self::#variant_names)),*,
                    _ => Err(format!("Invalid {} value: {}", stringify!(#enum_name), s))
                }
            }
        }

        impl std::convert::TryFrom<&str> for #enum_name {
            type Error = String;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::from_str(value)
            }
        }

        impl sqlx::Encode<'_, sqlx::Postgres> for #enum_name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                let s = self.to_string();
                <String as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
            }
        }

        impl sqlx::Decode<'_, sqlx::Postgres> for #enum_name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'_>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let s = value.as_str()?;
                Self::from_str(s).map_err(|e| sqlx::error::BoxDynError::from(
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e)
                ))
            }
        }

        impl sqlx::Type<sqlx::Postgres> for #enum_name {
            fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
                <String as sqlx::Type<sqlx::Postgres>>::type_info()
            }
        }
    };

    gen.into()
}
