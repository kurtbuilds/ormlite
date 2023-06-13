use syn::{Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed};

pub trait DeriveInputExt {
    fn fields(&self) -> syn::punctuated::Iter<Field>;
}

impl DeriveInputExt for DeriveInput {
    fn fields(&self) -> syn::punctuated::Iter<Field> {
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

pub trait FieldExt {
    fn name(&self) -> String;
}

impl FieldExt for Field {
    fn name(&self) -> String {
        self.ident.as_ref().unwrap().to_string().replace("r#", "")
    }
}
