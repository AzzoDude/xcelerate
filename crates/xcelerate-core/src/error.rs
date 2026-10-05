use thiserror::Error;

/// Errors raised by the CDP transport layer.
#[derive(Debug, Error)]
pub enum Error {
    #[error("WebSocket error: {0}")]
    Ws(String),

    #[error("JSON error: {0}")]
    Serde(String),

    #[error("CDP Error {code}: {message}")]
    Cdp { code: i32, message: String },

    #[error("BiDi error {error}: {message}")]
    Bidi { error: String, message: String },

    #[error("Internal channel error")]
    Internal,
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Ws(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
