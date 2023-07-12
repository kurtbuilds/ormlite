use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Context, Error, Result};
use clap::Parser;
use sqlmo::{Dialect, migrate::Statement, Migration, Schema, ToSql};
use sqlmo_sqlx::FromPostgres;
use time::macros::format_description;
use time::OffsetDateTime as DateTime;
use tokio::runtime::Runtime;

use ormlite::Row;
use ormlite::Acquire;
use ormlite::postgres::PgConnection;
use ormlite_core::config;
use ormlite_core::config::get_var_model_folders;
use ormlite_core::schema::TryFromOrmlite;

use crate::util::{create_connection, create_runtime};

const GET_MIGRATIONS_QUERY: &str = "SELECT
version || '_' || description AS name
, version
, description
FROM _sqlx_migrations
ORDER BY version ASC
";

// Not using FromRow because ormlite requires MODEL_FOLDERS to be set when compiling macros,
// and it's not set if we're compiling the CLI.
/// Compare migrations using version (see PartialEq).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MigrationMetadata {
    pub version: i64,
    /// For fs migrations, the name will be a file stem (for reversible, it'll be `<version>_<description>.<up/down>`).
    pub name: String,
    pub description: String,
}

impl MigrationMetadata {
    pub(crate) fn migration_type(&self) -> MigrationType {
        if self.name.ends_with(".up") {
            MigrationType::Up
        } else if self.name.ends_with(".down") {
            MigrationType::Down
        } else {
            MigrationType::Simple
        }
    }

    pub fn version_str(&self) -> String {
        self.version.to_string()
    }
}

#[derive(Debug, PartialEq)]
pub enum MigrationType {
    Simple,
    Up,
    Down,
}

impl MigrationType {
    pub fn extension(&self) -> &'static str {
        use MigrationType::*;
        match self {
            Simple => "sql",
            Up => "up.sql",
            Down => "down.sql",
        }
    }
}

#[derive(Parser, Debug)]
pub struct Migrate {
    name: String,

    /// Create an empty migration, don't attempt to infer changes
    #[clap(long)]
    empty: bool,

    /// Create an empty migration, don't attempt to infer changes
    #[clap(long, short)]
    reversible: bool,

    /// Instead of generating a migration, print the generated statements. Mostly useful for debugging.
    #[clap(long, short)]
    dry: bool,

    #[clap(long, short)]
    pub(crate) verbose: bool,
}


fn create_migration(folder: &Path, file_name: String, migration: MigrationType, content: &str) -> Result<()> {
    let file_path = folder.join(file_name).with_extension(migration.extension());

    let mut file = File::create(&file_path).context("Failed to create file")?;
    file.write_all(content.as_bytes())
        .context("Could not write to file")?;
    eprintln!("{}: Created migration", file_path.display());
    Ok(())
}

/// Migrations are sorted asc. Last is most recent.
pub fn get_executed_migrations(runtime: &Runtime, conn: &mut PgConnection) -> Result<Vec<MigrationMetadata>> {
    let migrations = runtime.block_on(ormlite::query(GET_MIGRATIONS_QUERY)
        .fetch_all(conn))?;
    let migrations = migrations.into_iter().map(|m: ormlite::postgres::PgRow| {
        let name: String = m.get("name");
        let version: i64 = m.get("version");
        let description: String = m.get("description");
        MigrationMetadata {
            name: name.to_string(),
            version,
            description,
        }
    }).collect();
    Ok(migrations)
}

/// Migrations are sorted asc. Last is most recent.
pub fn get_pending_migrations(folder: &Path) -> Result<Vec<MigrationMetadata>> {
    let mut migrations = fs::read_dir(folder)?
        .filter_map(|e| e.ok())
        .map(|f| f.path())
        .filter(|f| f.is_file() && f.extension().map(|e| e == "sql").unwrap_or(false))
        .map(|f| {
            let name = f.file_stem().unwrap().to_str().unwrap().to_string();
            let (version, description) = name.split_once('_').unwrap();
            let version = version.parse()?;
            let description = description.to_string();
            Ok::<_, Error>(MigrationMetadata {
                name,
                version,
                description,
            })
        }).collect::<Result<Vec<_>, _>>()?;
    migrations.sort();
    Ok(migrations)
}

// Returns the type of migration environment, either None (any), Simple, or Up (it doesn't return Down)
fn check_for_pending_migrations(folder: &Path, runtime: &Runtime, conn: &mut PgConnection) -> Result<Option<MigrationType>> {
    let executed = get_executed_migrations(runtime, conn)?;
    let pending = get_pending_migrations(folder)?;

    if executed.len() < pending.len() {
        return Err(anyhow!("Pending migrations are not in sync with the database. Please run `ormlite up` first."));
    }
    for (executed, pending) in executed.iter().zip(pending.iter()) {
        if executed != pending {
            eprintln!("WARNING: Your file system migrations do not match the database. Have {} on the file system but {} in the database.", pending.name, executed.name);
        }
    }

    Ok(pending.first().map(|m| m.migration_type()))
}

fn check_reversible_compatibility(reversible: bool, migration_environment: Option<MigrationType>) -> Result<()> {
    if let Some(migration_environment) = migration_environment {
        if reversible && migration_environment == MigrationType::Simple {
            return Err(anyhow!("You cannot mix reversible and non-reversible migrations"));
        } else if !reversible && migration_environment != MigrationType::Simple {
            return Err(anyhow!("You cannot mix reversible and non-reversible migrations"));
        }
    }
    Ok(())
}

#[allow(unused_variables)]
fn experimental_modifications_to_schema(schema: &mut Schema) -> Result<()> {
    #[allow(unused_imports)]
    use sqlmo::Type;
    for table in &mut schema.tables {
        for column in &mut table.columns {
            #[cfg(feature = "experimental-sid")]
            match &column.typ {
                Type::Other(z) if z == "Sid" => {
                    column.typ = Type::Uuid;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn autogenerate_migration(codebase_path: &[&Path], runtime: &Runtime, conn: &mut PgConnection, opts: &Migrate) -> Result<Migration> {
    let mut current = runtime.block_on(Schema::try_from_postgres(conn, "public"))?;
    current.tables.retain(|t| t.name != "_sqlx_migrations");

    let mut desired = Schema::try_from_ormlite_project(codebase_path)?;
    experimental_modifications_to_schema(&mut desired)?;

    let migration = current.migrate_to(desired, &sqlmo::MigrationOptions {
        debug: opts.verbose,
    })?;
    Ok(migration)
}


impl Migrate {
    pub fn run(self) -> Result<()> {
        let runtime = create_runtime();

        let folder = config::get_var_migration_folder();
        let url = config::get_var_database_url();

        let mut conn = create_connection(&url, &runtime)?;
        let conn = runtime.block_on(conn.acquire())?;

        fs::create_dir_all(&folder)?;
        let migration_environment = check_for_pending_migrations(&folder, &runtime, conn)?;
        check_reversible_compatibility(self.reversible, migration_environment)?;

        let migration = if self.empty {
            None
        } else {
            let folder_paths = get_var_model_folders();
            let folder_paths = folder_paths.iter().map(|p| p.as_path()).collect::<Vec<_>>();
            let migration = autogenerate_migration(&folder_paths, &runtime, conn, &self)?;

            for d in &migration.debug_results {
                tracing::debug!(table=d.table_name(), "Table is identical");
            }

            if self.dry {
                for statement in migration.statements {
                    println!("{};", statement.to_sql(Dialect::Postgres));
                }
                return Ok(());
            }
            Some(migration)
        };

        fs::create_dir_all(&folder).context("Unable to create migrations directory")?;

        let dt = DateTime::now_utc();
        let mut file_name = dt.format(format_description!("[year][month][day][hour][minute][second]"))?;
        file_name.push('_');
        file_name.push_str(&self.name);
        let migration_body = migration.as_ref().map(|m| {
            m.statements.iter()
                .inspect(|&f| { dbg!(f); })
                .map(|s| s.to_sql(Dialect::Postgres))
                .collect::<Vec<_>>()
                .join(";\n")
        }).unwrap_or_default();
        if self.reversible {
            create_migration(&folder, file_name.clone(), MigrationType::Up, &migration_body)?;
            create_migration(&folder, file_name.clone(), MigrationType::Down, "")?;
        } else {
            create_migration(&folder, file_name.clone(), MigrationType::Simple, &migration_body)?;
        }
        if let Some(migration) = migration {
            if migration.statements.is_empty() {
                println!("No changes were detected. Generated an empty migration file.");
            } else {
                println!("It auto-generated the following actions:");
                for statement in &migration.statements {
                    match statement {
                        Statement::CreateTable(t) => println!("Create table {} with columns: {}", &t.name, t.columns.iter().map(|c| c.name.to_string()).collect::<Vec<_>>().join(", ")),
                        Statement::CreateIndex(s) => println!("Create index {} on {}", &s.name, &s.table),
                        Statement::AlterTable(s) => println!("Alter table {}", &s.name),
                    }
                }
            }
        }
        Ok(())
    }
}