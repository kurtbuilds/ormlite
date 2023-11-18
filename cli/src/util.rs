use anyhow::Error;
use tokio::runtime::Runtime;

use ormlite::Connection;
use ormlite::postgres::PgConnection;

pub(crate) fn create_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
}

pub fn create_connection(url: &str, runtime: &Runtime) -> anyhow::Result<PgConnection> {
    let conn = runtime.block_on(ormlite::postgres::PgConnection::connect(url))?;
    Ok(conn)
}

pub trait CommandSuccess {
    fn ok_or(&mut self, message: &str) -> Result<(), anyhow::Error>;
}

impl CommandSuccess for std::process::Command {
    fn ok_or(&mut self, message: &str) -> Result<(), anyhow::Error> {
        let status = self.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::msg(message.to_string()))
        }
    }
}

impl CommandSuccess for std::process::Output {
    fn ok_or(&mut self, message: &str) -> Result<(), anyhow::Error> {
        if self.status.success() {
            Ok(())
        } else {
            Err(Error::msg(message.to_string()))
        }
    }
}