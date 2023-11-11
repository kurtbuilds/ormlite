#![allow(non_snake_case)]

mod attr;
mod metadata;
mod error;
mod ext;
mod syndecode;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Context;
use syn::{DeriveInput, Item};
use ignore::Walk;

use syndecode::{Attributes2};
pub use metadata::*;
pub use attr::*;
pub use error::*;
pub use ext::*;

#[derive(Default, Debug)]
pub struct LoadOptions {
    pub verbose: bool,
}

/// This is an intermediate representation of the schema.
///
pub struct OrmliteSchema {
    pub tables: Vec<ModelMetadata>,
    // map of rust structs (e.g. enums) to database encodings.
    // note that these are not bona fide postgres types.
    pub type_reprs: HashMap<String, String>,
}

type WithAttr<T> = (T, Attributes2);

struct Intermediate {
    model_structs: Vec<WithAttr<syn::ItemStruct>>,
    type_structs: Vec<WithAttr<syn::ItemStruct>>,
    type_enums: Vec<WithAttr<syn::ItemEnum>>,
}

impl Intermediate {
    fn into_models_and_types(self) -> (impl Iterator<Item=WithAttr<syn::ItemStruct>>, impl Iterator<Item=WithAttr<String>>) {
        let models = self.model_structs.into_iter();
        let types = self.type_structs.into_iter().map(|(s, a)| (s.ident.to_string(), a))
            .chain(self.type_enums.into_iter().map(|(e, a)| (e.ident.to_string(), a)));
        (models, types)
    }

    fn from_with_opts(value: syn::File) -> Self {
        let mut model_structs = Vec::new();
        let mut type_structs = Vec::new();
        let mut type_enums = Vec::new();
        for item in value.items {
            match item {
                Item::Struct(s) => {
                    let attrs = Attributes2::from(s.attrs.as_slice());
                    if attrs.has_derive("Model") {
                        tracing::debug!(model=%s.ident.to_string(), "Found");
                        model_structs.push((s, attrs));
                    } else if attrs.has_derive("Type") {
                        tracing::debug!(r#type=%s.ident.to_string(), "Found");
                        type_structs.push((s, attrs));
                    } else if attrs.has_derive("ManualType") {
                        tracing::debug!(r#type=%s.ident.to_string(), "Found");
                        type_structs.push((s, attrs));
                    }
                }
                Item::Enum(e) => {
                    let attrs = Attributes2::from(e.attrs.as_slice());
                    if attrs.has_derive("Type") || attrs.has_derive("ManualType") {
                        tracing::debug!(r#type=%e.ident.to_string(), "Found");
                        type_enums.push((e, attrs));
                    }
                }
                _ => {}
            }
        }
        Self {
            model_structs,
            type_structs,
            type_enums,
        }
    }
}

pub fn schema_from_filepaths(paths: &[&Path]) -> anyhow::Result<OrmliteSchema> {
    let invalid_paths = paths.iter().filter(|p| fs::metadata(p).is_err()).collect::<Vec<_>>();
    if !invalid_paths.is_empty() {
        for path in invalid_paths {
            tracing::error!(path=path.display().to_string(), "Does not exist");
        }
        anyhow::bail!("Provided paths that did not exist.");
    }

    let walk = paths.iter().flat_map(Walk::new);

    let walk = walk.map(|e| e.unwrap())
        .filter(|e| e.path().extension().map(|e| e == "rs")
            .unwrap_or(false))
        .map(|e| e.into_path())
        .chain(paths.iter()
            .filter(|p| p.ends_with(".rs"))
            .map(|p| p.to_path_buf())
        );

    let mut tables = vec![];
    let mut type_aliases = HashMap::new();
    for entry in walk {
        let contents = fs::read_to_string(&entry)
            .context(format!("failed to read file: {}", entry.display()))?;
        tracing::debug!(file=entry.display().to_string(), "Checking for Model, Type, ManualType derive attrs");
        if !(contents.contains("Model") || contents.contains("Type") || contents.contains("ManualType")) {
            continue;
        }
        let ast = syn::parse_file(&contents)
            .context(format!("Failed to parse file: {}", entry.display()))?;
        let intermediate = Intermediate::from_with_opts(ast);
        let (models, types) = intermediate.into_models_and_types();

        for (item, _attrs) in models {
            let derive: DeriveInput = item.into();
            let table = ModelMetadata::from_derive(&derive)
                .context(format!("Failed to parse model: {}", derive.ident.to_string()))?;
            tables.push(table);
        }

        for (name, attrs) in types {
            let ty = attrs.repr.unwrap_or_else(|| "String".to_string());
            type_aliases.insert(name, ty);
        }
    }
    Ok(OrmliteSchema {
        tables,
        type_reprs: type_aliases,
    })
}
