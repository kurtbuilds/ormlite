pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("tokenization error: {0}")]
    TokenizationError(#[from] sqlparser::tokenizer::TokenizerError),

    #[error(transparent)]
    OrmliteError(String),
}