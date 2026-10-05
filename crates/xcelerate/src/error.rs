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

    #[error("unsupported: {0}")]
    Unsupported(String),

    #[error("plugin error: {0}")]
    Plugin(String),
}

impl From<xcelerate_core::Error> for XcelerateError {
    fn from(e: xcelerate_core::Error) -> Self {
        use xcelerate_core::Error as CoreError;
        match e {
            CoreError::Ws(message) => Self::WsError(message),
            CoreError::Serde(message) => Self::SerdeError(message),
            CoreError::Cdp { code, message } => Self::CdpResponseError { code, message },
            CoreError::Bidi { error, message } => Self::WsError(format!("{error}: {message}")),
            CoreError::Internal => Self::InternalError,
        }
    }
}

impl From<xcelerate_plugin_api::PluginError> for XcelerateError {
    fn from(e: xcelerate_plugin_api::PluginError) -> Self {
        use xcelerate_plugin_api::PluginError as ApiError;
        match e {
            ApiError::NotFound(message) => Self::NotFound(message),
            ApiError::Unsupported(message) => Self::Unsupported(message),
            ApiError::Message(message) => Self::Plugin(message),
        }
    }
}

impl From<XcelerateError> for xcelerate_plugin_api::PluginError {
    fn from(e: XcelerateError) -> Self {
        xcelerate_plugin_api::PluginError::Message(e.to_string())
    }
}

impl From<reqwest::Error> for XcelerateError {
    fn from(e: reqwest::Error) -> Self {
        Self::HttpError(e.to_string())
    }
}

impl From<serde_json::Error> for XcelerateError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerdeError(e.to_string())
    }
}

pub type XcelerateResult<T> = Result<T, XcelerateError>;
