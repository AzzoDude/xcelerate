use thiserror::Error;

/// Errors raised by the stealth layer.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    NotFound(String),

    #[error("internal error")]
    Internal,
}

pub type Result<T> = std::result::Result<T, Error>;
