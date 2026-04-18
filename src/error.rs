use thiserror::Error;

#[derive(Debug, Error)]
pub enum XcelerateError {
    #[error("WebSocket error: {0}")]
    WsError(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("JSON error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("CDP Error {code}: {message}")]
    CdpResponseError { code: i32, message: String },
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Target not found: {0}")]
    NotFound(String),
    
    #[error("Internal channel error")]
    InternalError,
}

pub type XcelerateResult<T> = Result<T, XcelerateError>;
