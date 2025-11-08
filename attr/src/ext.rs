use syn::{Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed};

pub trait DeriveInputExt {
    fn fields(&self) -> syn::punctuated::Iter<'_, Field>;
}

impl DeriveInputExt for DeriveInput {
    fn fields(&self) -> syn::punctuated::Iter<'_, Field> {
        let fields = match &self.data {
            Data::Struct(DataStruct { ref fields, .. }) => fields,
            _ => panic!("#[ormlite] can only be used on structs"),
        };
        let fields = match fields {
            Fields::Named(FieldsNamed { named, .. }) => named,
            _ => panic!("#[ormlite] can only be used on structs with named fields"),
        };
        fields.iter()
    }
}
