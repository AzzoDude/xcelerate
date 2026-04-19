use thiserror::Error;

#[derive(Debug, Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum XcelerateError {
    #[error("WebSocket error: {0}")]
    WsError(String),
    
    #[error("JSON error: {0}")]
    SerdeError(String),
    
    #[error("CDP Error {code}: {message}")]
    CdpResponseError { code: i32, message: String },
    
    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Target not found: {0}")]
    NotFound(String),
    
    #[error("Internal channel error")]
    InternalError,
}

impl From<tokio_tungstenite::tungstenite::Error> for XcelerateError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WsError(e.to_string())
    }
}

impl From<serde_json::Error> for XcelerateError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerdeError(e.to_string())
    }
}

impl From<reqwest::Error> for XcelerateError {
    fn from(e: reqwest::Error) -> Self {
        Self::HttpError(e.to_string())
    }
}

pub type XcelerateResult<T> = Result<T, XcelerateError>;
