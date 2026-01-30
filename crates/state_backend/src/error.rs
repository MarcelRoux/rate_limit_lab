use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("redis error: {0}")]
    Redis(#[from] fred::error::Error),

    #[error("invalid backend response")]
    InvalidResponse,
}
