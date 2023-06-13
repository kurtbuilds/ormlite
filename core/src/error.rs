pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    TokenizationError(#[from] sqlparser::tokenizer::TokenizerError),

    #[error("{0}")]
    OrmliteError(String),
}
