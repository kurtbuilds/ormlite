use anyhow::Result as AnyResult;
use std::{fs::File, io::Read, path::Path};
pub use ormlite_core::config::Config;

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


