pub type CoreResult<T, E = CoreError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    TokenizationError(#[from] sqlparser::tokenizer::TokenizerError),

    #[error("{0}")]
    OrmliteError(String),
}
