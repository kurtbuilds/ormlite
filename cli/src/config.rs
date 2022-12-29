use std::env::var;
use std::path::PathBuf;
use std::str::FromStr;

const MIGRATION_FOLDER: &str = "migrations";
const MIGRATION_TABLE: &str = "_sqlx_migrations";
const MIGRATION_SNAPSHOT_FOLDER: &str = "migrations/snapshot";

pub fn get_var_migration_folder() -> PathBuf {
    let folder = var("MIGRATION_FOLDER").unwrap_or_else(|_| MIGRATION_FOLDER.to_string());
    PathBuf::from_str(&folder).unwrap()
}

pub fn get_var_snapshot_folder() -> PathBuf {
    let folder = var("MIGRATION_BACKUP_FOLDER").unwrap_or_else(|_| MIGRATION_SNAPSHOT_FOLDER.to_string());
    PathBuf::from_str(&folder).unwrap()
}

pub fn get_var_database_url() -> String {
    var("DATABASE_URL").expect("DATABASE_URL must be set")
}
