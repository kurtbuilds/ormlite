use anyhow::Result as AnyResult;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

static CONFIG_REL_PATHS: [&str; 2] = [".ormlite/config.toml", ".ormlite.toml"];

pub fn load_config() -> AnyResult<Config> {
    let home_dir = dirs::home_dir().unwrap();
    let config_dir = dirs::config_dir().unwrap();
    let search_paths: &[&str] = &[".", "..", config_dir.to_str().unwrap(), home_dir.to_str().unwrap()];
    for p in search_paths {
        for rel_path in &CONFIG_REL_PATHS {
            let path = format!("{}/{}", p, rel_path);
            let path = Path::new(&path);
            if path.exists() {
                return read(path);
            }
        }
    }
    Ok(Config::default())
}

pub fn read(path: impl AsRef<Path>) -> AnyResult<Config> {
    let path = path.as_ref();
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let config: Config = toml::from_str(&buf)?;
    Ok(config)
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub table: Table,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Table {
    /// When auto detecting foreign keys, use this aliases
    /// For example, if you have a table organization, but the foreign key is org_id,
    /// you'd define the alias as "org" => "organization"
    pub aliases: IndexMap<String, String>,
}
