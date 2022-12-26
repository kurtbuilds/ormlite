use std::path::Path;
use crate::Schema;
use anyhow::Result;
use ignore::Walk;

impl Schema {
    pub fn try_from_ormlite_project(path: &Path) -> Result<Self> {
        for entry in Walk(path).filter_map(|e| e.ok()) {
            println!("{}", entry.path().display());
        }
        Self {
            tables: vec![],
        }
    }
}