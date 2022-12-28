use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct SyndecodeError(pub(crate) String);

impl Display for SyndecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for SyndecodeError {}
