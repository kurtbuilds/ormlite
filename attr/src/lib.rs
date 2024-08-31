#![allow(non_snake_case)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::Context;
use ignore::Walk;
use syn::{DeriveInput, Item};

use crate::derive::DeriveParser;
use crate::repr::Repr;
pub use attr::*;
pub use error::*;
pub use ext::*;
pub use ident::*;
pub use metadata::*;
pub use ttype::*;

mod attr;
mod cfg_attr;
mod derive;
mod error;
mod ext;
mod ident;
mod metadata;
mod repr;
pub mod ttype;

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

struct Intermediate {
    model_structs: Vec<syn::ItemStruct>,
    type_structs: Vec<(syn::ItemStruct, Option<Repr>)>,
    type_enums: Vec<(syn::ItemEnum, Option<Repr>)>,
}

impl Intermediate {
    fn into_models_and_types(
        self,
    ) -> (
        impl Iterator<Item = syn::ItemStruct>,
        impl Iterator<Item = (String, Option<Repr>)>,
    ) {
        let models = self.model_structs.into_iter();
        let types = self
            .type_structs
            .into_iter()
            .map(|(s, a)| (s.ident.to_string(), a))
            .chain(self.type_enums.into_iter().map(|(e, a)| (e.ident.to_string(), a)));
        (models, types)
    }

    fn from_file(value: syn::File) -> Self {
        let mut model_structs = Vec::new();
        let mut type_structs = Vec::new();
        let mut type_enums = Vec::new();
        for item in value.items {
            match item {
                Item::Struct(s) => {
                    let attrs = DeriveParser::from_attributes(&s.attrs);
                    if attrs.has_derive("ormlite", "Model") {
                        tracing::debug!(model=%s.ident.to_string(), "Found");
                        model_structs.push(s);
                    } else if attrs.has_derive2(&["ormlite", "sqlx"], "Type") {
                        tracing::debug!(r#type=%s.ident.to_string(), "Found");
                        let repr = Repr::from_attributes(&s.attrs);
                        type_structs.push((s, repr));
                    } else if attrs.has_derive("ormlite", "ManualType") {
                        tracing::debug!(r#type=%s.ident.to_string(), "Found");
                        let repr = Repr::from_attributes(&s.attrs);
                        type_structs.push((s, repr));
                    }
                }
                Item::Enum(e) => {
                    let attrs = DeriveParser::from_attributes(&e.attrs);
                    if attrs.has_derive("ormlite", "Type") || attrs.has_derive("ormlite", "ManualType") {
                        tracing::debug!(r#type=%e.ident.to_string(), "Found");
                        let repr = Repr::from_attributes(&e.attrs);
                        type_enums.push((e, repr));
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
    let cwd = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| env::current_dir())
        .expect("Failed to get current directory for schema");
    let paths = paths.iter().map(|p| cwd.join(p)).collect::<Vec<_>>();
    let invalid_paths = paths.iter().filter(|p| fs::metadata(p).is_err()).collect::<Vec<_>>();
    if !invalid_paths.is_empty() {
        for path in &invalid_paths {
            tracing::error!(path = path.display().to_string(), "Does not exist");
        }
        let paths = invalid_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        anyhow::bail!("Provided paths that did not exist: {}", paths);
    }

    let walk = paths.iter().flat_map(Walk::new);

    let walk = walk
        .map(|e| e.unwrap())
        .filter(|e| e.path().extension().map(|e| e == "rs").unwrap_or(false))
        .map(|e| e.into_path())
        .chain(paths.iter().filter(|p| p.ends_with(".rs")).map(|p| p.to_path_buf()));

    let mut tables = vec![];
    let mut type_aliases = HashMap::new();
    for entry in walk {
        let contents = fs::read_to_string(&entry).context(format!("failed to read file: {}", entry.display()))?;
        tracing::debug!(
            file = entry.display().to_string(),
            "Checking for Model, Type, ManualType derive attrs"
        );
        if !(contents.contains("Model") || contents.contains("Type") || contents.contains("ManualType")) {
            continue;
        }
        let ast = syn::parse_file(&contents).context(format!("Failed to parse file: {}", entry.display()))?;
        let intermediate = Intermediate::from_file(ast);
        let (models, types) = intermediate.into_models_and_types();

        for item in models {
            let derive: DeriveInput = item.into();
            let table = ModelMetadata::from_derive(&derive)
                .context(format!("Failed to parse model: {}", derive.ident.to_string()))?;
            tables.push(table);
        }

        for (name, repr) in types {
            let ty = repr.map(|s| s.to_string()).unwrap_or_else(|| "String".to_string());
            type_aliases.insert(name, ty);
        }
    }
    Ok(OrmliteSchema {
        tables,
        type_reprs: type_aliases,
    })
}
