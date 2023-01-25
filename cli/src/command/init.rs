


use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use ormlite::Acquire;


use ormlite::Executor;

use ormlite_core::config::{get_var_database_url};
use crate::util::{create_connection, create_runtime};

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
pub struct Init {
}

impl Init {
    pub fn run(self) -> Result<()> {
        let runtime = create_runtime();
        let url = get_var_database_url();
        let mut conn = create_connection(&url, &runtime)?;
        let conn = runtime.block_on(conn.acquire())?;

        runtime.block_on(conn.execute(INIT_QUERY))?;

        eprintln!("{} Initialized database at {}", "SUCCESS".green(), url);
        Ok(())
    }
}