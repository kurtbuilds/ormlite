use anyhow::Error;

pub(crate) fn create_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
}

pub trait CommandSuccess {
    fn ok_or(&mut self, message: &str) -> Result<(), Error>;
}

impl CommandSuccess for std::process::Command {
    fn ok_or(&mut self, message: &str) -> Result<(), Error> {
        let status = self.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::msg(message.to_string()))
        }
    }
}

impl CommandSuccess for std::process::Output {
    fn ok_or(&mut self, message: &str) -> Result<(), Error> {
        if self.status.success() {
            Ok(())
        } else {
            Err(Error::msg(message.to_string()))
        }
    }
}
