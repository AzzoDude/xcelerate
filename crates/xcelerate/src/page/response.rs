//! Network response body capture (capability B).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

impl Page {
    /// Fetches a response body by CDP `requestId` and returns it as text.
    ///
    /// Calls `Network.getResponseBody` on the page's session. When the CDP
    /// response is base64-encoded (binary payloads), the body is decoded back
    /// to a UTF-8 string before being returned. Returns [`XcelerateError::NotFound`]
    /// when the response carries no `body` field and
    /// [`XcelerateError::SerdeError`] when a payload cannot be decoded.
    pub async fn response_body(&self, request_id: String) -> XcelerateResult<String> {
        let value = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Network.getResponseBody",
                serde_json::json!({ "requestId": request_id.clone() }),
            )
            .await?;
        let body = value
            .get("body")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                XcelerateError::NotFound(format!("no response body for request {request_id}"))
            })?;
        let base64_encoded = value
            .get("base64Encoded")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        decode_body(body, base64_encoded)
    }

    /// Fetches a response body and returns the raw CDP result JSON.
    ///
    /// The returned string is the serialized `{ "body", "base64Encoded" }`
    /// object produced by `Network.getResponseBody`, with no decoding applied.
    /// Returns [`XcelerateError::SerdeError`] if the value cannot be serialized.
    pub async fn response_body_json(&self, request_id: String) -> XcelerateResult<String> {
        let value = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Network.getResponseBody",
                serde_json::json!({ "requestId": request_id }),
            )
            .await?;
        serde_json::to_string(&value).map_err(|e| XcelerateError::SerdeError(e.to_string()))
    }
}

/// Decodes a CDP response body that may be base64-encoded.
///
/// Pure helper mirroring the `{ body, base64Encoded }` shape: plain bodies are
/// passed through unchanged, base64 bodies are decoded and UTF-8 validated.
pub(crate) fn decode_body(body: &str, base64_encoded: bool) -> XcelerateResult<String> {
    if !base64_encoded {
        return Ok(body.to_string());
    }
    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body)
        .map_err(|e| XcelerateError::SerdeError(format!("Base64 decode failed: {e}")))?;
    String::from_utf8(bytes)
        .map_err(|e| XcelerateError::SerdeError(format!("UTF-8 decode failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::decode_body;

    #[test]
    fn decode_body_passes_through_plain_text() {
        assert_eq!(
            decode_body("hello", false).expect("plain passthrough"),
            "hello"
        );
    }

    #[test]
    fn decode_body_decodes_base64() {
        assert_eq!(
            decode_body("aGVsbG8=", true).expect("base64 decode"),
            "hello"
        );
    }
}
