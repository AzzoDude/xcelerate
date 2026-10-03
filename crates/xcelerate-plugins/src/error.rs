use thiserror::Error;

/// Errors raised by the plugin support layer (binary patching, process control,
/// and the first-party plugins).
#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    NotFound(String),

    #[error("internal error")]
    Internal,
}

pub type Result<T> = std::result::Result<T, Error>;
