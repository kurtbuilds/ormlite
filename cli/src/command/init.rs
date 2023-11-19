use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use ormlite::{Acquire, Connection};


use ormlite::Executor;
use ormlite::postgres::PgConnection;

use ormlite_core::config::{get_var_database_url};
use crate::util::{create_runtime};

const INIT_QUERY: &str = r#"
CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);
"#;

#[derive(Parser, Debug)]
pub struct Init {}

impl Init {
    pub fn run(self) -> Result<()> {
        let runtime = create_runtime();
        let url = get_var_database_url();
        runtime.block_on(async {
            let mut conn = PgConnection::connect(&url).await?;
            let conn = conn.acquire().await?;
            let table_exists = conn.execute("select 1 from _sqlx_migrations limit 1").await.is_ok();
            if table_exists {
                eprintln!("{} Database {} already initialized. No actions taken.", "SUCCESS".green(), url);
                return Ok(());
            }
            conn.execute(INIT_QUERY).await?;
            eprintln!("{} Initialized database at {}", "SUCCESS".green(), url);
            Ok(())
        })
    }
}