//! Network-interception background pump for [`super::Page`].
//!
//! The pump answers `Fetch.requestPaused` from the page's stored route rules and
//! `Fetch.authRequired` from its stored credentials. The rule type itself lives
//! in the parent module because it is part of the page's state.

use std::sync::Arc;

use crate::CdpClient;

use super::RouteRule;

/// Matches a Playwright-style glob (`*` wildcard) or a plain substring.
fn pattern_matches(pattern: &str, url: &str) -> bool {
    if pattern.is_empty() || pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return url.contains(pattern);
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut rest = url;
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if index == 0 {
            if !rest.starts_with(part) {
                return false;
            }
            rest = &rest[part.len()..];
        } else if index == parts.len() - 1 {
            if !rest.ends_with(part) {
                return false;
            }
        } else if let Some(position) = rest.find(part) {
            rest = &rest[position + part.len()..];
        } else {
            return false;
        }
    }
    true
}

/// Background pump: answers `Fetch.requestPaused` from the stored rules and
/// handles `Fetch.authRequired` from the stored credentials.
pub(super) async fn run_interception(
    client: Arc<CdpClient>,
    session_id: String,
    routes: Arc<tokio::sync::Mutex<Vec<RouteRule>>>,
    requests: Arc<tokio::sync::Mutex<Vec<serde_json::Value>>>,
    credentials: Arc<tokio::sync::Mutex<Option<(String, String)>>>,
) {
    let _ = client
        .execute_raw_with_session(
            Some(&session_id),
            "Fetch.enable",
            serde_json::json!({ "patterns": [{ "urlPattern": "*" }] }),
        )
        .await;

    let mut receiver = client.subscribe();
    loop {
        let value = match receiver.recv().await {
            Ok(value) => value,
            // A slow consumer only misses events; keep pumping.
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        };
        if value.get("sessionId").and_then(|s| s.as_str()) != Some(session_id.as_str()) {
            continue;
        }
        let method = value.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = value
            .get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        match method {
            "Fetch.requestPaused" => {
                let request_id = params
                    .get("requestId")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let request = params.get("request").cloned().unwrap_or_default();
                let url = request
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or_default()
                    .to_string();
                requests.lock().await.push(request);
                let rule = {
                    let guard = routes.lock().await;
                    guard
                        .iter()
                        .find(|rule| pattern_matches(&rule.pattern, &url))
                        .cloned()
                };
                let result = match rule {
                    Some(rule) if rule.action == "abort" => {
                        client
                            .execute_raw_with_session(
                                Some(&session_id),
                                "Fetch.failRequest",
                                serde_json::json!({ "requestId": request_id, "errorReason": "Aborted" }),
                            )
                            .await
                    }
                    Some(rule) if rule.action == "fulfill" => {
                        use base64::{Engine as _, engine::general_purpose};
                        let body = general_purpose::STANDARD.encode(rule.body.unwrap_or_default());
                        let content_type = rule
                            .content_type
                            .unwrap_or_else(|| "text/html".to_string());
                        client
                            .execute_raw_with_session(
                                Some(&session_id),
                                "Fetch.fulfillRequest",
                                serde_json::json!({
                                    "requestId": request_id,
                                    "responseCode": 200,
                                    "body": body,
                                    "responseHeaders": [{ "name": "Content-Type", "value": content_type }]
                                }),
                            )
                            .await
                    }
                    _ => {
                        client
                            .execute_raw_with_session(
                                Some(&session_id),
                                "Fetch.continueRequest",
                                serde_json::json!({ "requestId": request_id }),
                            )
                            .await
                    }
                };
                let _ = result;
            }
            "Fetch.authRequired" => {
                let request_id = params
                    .get("requestId")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let creds = credentials.lock().await.clone();
                let payload = match creds {
                    Some((username, password)) => serde_json::json!({
                        "requestId": request_id,
                        "authChallengeResponse": {
                            "response": "ProvideCredentials",
                            "username": username,
                            "password": password
                        }
                    }),
                    None => serde_json::json!({
                        "requestId": request_id,
                        "authChallengeResponse": { "response": "Default" }
                    }),
                };
                let _ = client
                    .execute_raw_with_session(Some(&session_id), "Fetch.continueWithAuth", payload)
                    .await;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pattern_matches;

    #[test]
    fn empty_and_star_match_everything() {
        assert!(pattern_matches("", "https://example.com/a.png"));
        assert!(pattern_matches("*", "https://example.com/a.png"));
    }

    #[test]
    fn plain_pattern_is_a_substring_match() {
        assert!(pattern_matches("example", "https://example.com/a.png"));
        assert!(!pattern_matches("absent", "https://example.com/a.png"));
    }

    #[test]
    fn leading_wildcard_matches_suffix() {
        assert!(pattern_matches("*.png", "https://example.com/a.png"));
        assert!(!pattern_matches("*.png", "https://example.com/a.jpg"));
    }

    #[test]
    fn trailing_wildcard_matches_prefix() {
        assert!(pattern_matches("https://*", "https://example.com"));
        assert!(!pattern_matches("https://a*", "https://b"));
    }

    #[test]
    fn middle_wildcard_spans_a_gap() {
        assert!(pattern_matches("a*c", "abc"));
        assert!(!pattern_matches("a*c", "abx"));
    }
}
