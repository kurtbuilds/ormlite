use crate::schema::schema_from_ormlite_project;
use crate::util::create_runtime;
use anyhow::Result;
use clap::Parser;
use ormlite::postgres::PgConnection;
use ormlite::{Acquire, Connection};
use ormlite_core::config::{get_var_database_url, get_var_model_folders};
use sql::Schema;
use sql_sqlx::FromPostgres;

#[derive(Parser, Debug)]
pub struct Info {
    /// The name of the migration
    #[clap(long, short)]
    url: bool,

    #[clap(long, short)]
    table: Option<String>,
}

impl Info {
    pub fn run(self) -> Result<()> {
        // let folder = get_var_migration_folder();
        let runtime = create_runtime();
        let c = crate::config::load_config()?;
        let mut current = if self.url {
            let url = get_var_database_url();
            let mut conn = runtime.block_on(PgConnection::connect(&url))?;
            let conn = runtime.block_on(conn.acquire())?;
            runtime.block_on(Schema::try_from_postgres(conn, "public"))?
        } else {
            let folder_paths = get_var_model_folders();
            let folder_paths = folder_paths.iter().map(|p| p.as_path()).collect::<Vec<_>>();
            schema_from_ormlite_project(&folder_paths, &c)?
        };
        if let Some(s) = self.table {
            current.tables.retain(|t| t.name == s);
        }
        for table in current.tables {
            eprintln!("Table: {}", table.name);
            for column in table.columns {
                let nullable = if column.nullable { " " } else { " NOT NULL " };
                let constraint = if let Some(constraint) = &column.constraint {
                    format!(" {:?}", constraint)
                } else {
                    "".to_string()
                };
                eprintln!("  {}: {:?}{}{}", column.name, column.typ, nullable, constraint)
            }
        }
        Ok(())
    }
}
