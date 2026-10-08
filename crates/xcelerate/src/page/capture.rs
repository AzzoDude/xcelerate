//! Media capture: record the media requests a page makes while it plays, then
//! reassemble them into a file.
//!
//! This is how a Media Source Extensions player — YouTube, Facebook, any
//! dash.js/hls.js page — is downloaded **without an external tool and without
//! deciphering the player's signing logic**: the page requests its own (already
//! signed) segments, and we replay those exact URLs. The player solved the
//! signature for us; we only need the bytes it asked for, in order.
//!
//! Limits are the same as any capture: the video must actually play, the signed
//! URLs expire, and separate audio and video are written to two files (muxing
//! two tracks into one container is a muxer's job).

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::CdpClient;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

/// Background pump: collects media responses until the task is aborted.
pub(crate) async fn run_capture(
    client: Arc<CdpClient>,
    session_id: String,
    entries: Arc<tokio::sync::Mutex<Vec<Value>>>,
) {
    let mut receiver = client.subscribe_session(&session_id);
    // requestId -> request URL, so a response we cannot map otherwise still
    // yields a URL.
    let mut pending: HashMap<String, String> = HashMap::new();
    loop {
        let value = match receiver.recv().await {
            Ok(value) => value,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        };
        if value.get("sessionId").and_then(Value::as_str) != Some(session_id.as_str()) {
            continue;
        }
        let method = value.get("method").and_then(Value::as_str).unwrap_or("");
        let params = value.get("params").cloned().unwrap_or(Value::Null);
        let request_id = params
            .get("requestId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if request_id.is_empty() {
            continue;
        }
        match method {
            "Network.requestWillBeSent" => {
                if let Some(url) = params.pointer("/request/url").and_then(Value::as_str) {
                    pending.insert(request_id, url.to_string());
                }
            }
            "Network.responseReceived" => {
                let mime = params
                    .pointer("/response/mimeType")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                let url = params
                    .pointer("/response/url")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| pending.get(&request_id).cloned());
                let Some(url) = url else { continue };
                let Some(kind) = media_kind(&mime, &url) else {
                    continue;
                };
                let mut entries = entries.lock().await;
                // A player can re-request a URL; keep each once, in first-seen order.
                if !entries.iter().any(|entry| entry["url"] == json!(url)) {
                    entries.push(json!({ "url": url, "mime": mime, "kind": kind }));
                }
            }
            _ => {}
        }
    }
}

/// Classifies a response as a media segment, if it is one.
fn media_kind(mime: &str, url: &str) -> Option<&'static str> {
    if mime.starts_with("video/") {
        return Some("video");
    }
    if mime.starts_with("audio/") {
        return Some("audio");
    }
    // Fall back to well-known segment URL shapes (some CDNs omit a media type).
    let url = url.to_ascii_lowercase();
    if url.contains("videoplayback") || url.contains(".m4s") || url.contains(".ts") {
        return Some("video");
    }
    None
}

impl Page {
    /// Starts capturing the media the page requests (MSE/HLS/DASH segments).
    ///
    /// Enables the CDP `Network` domain and spawns a background pump. Call this
    /// **before** the page starts playing so the init segment is captured too.
    /// If a capture is already running it is restarted.
    pub async fn start_media_capture(&self) -> XcelerateResult<()> {
        if let Some(handle) = self.capture_task.lock().await.take() {
            handle.abort();
        }
        self.capture_entries.lock().await.clear();
        self.client
            .execute_raw_with_session(Some(&self.session_id), "Network.enable", json!({}))
            .await?;
        let handle = tokio::spawn(run_capture(
            Arc::clone(&self.client),
            self.session_id.clone(),
            Arc::clone(&self.capture_entries),
        ));
        *self.capture_task.lock().await = Some(handle);
        Ok(())
    }

    /// Stops capturing. Entries collected so far are kept for [`Page::save_capture`].
    pub async fn stop_media_capture(&self) -> XcelerateResult<()> {
        if let Some(handle) = self.capture_task.lock().await.take() {
            handle.abort();
        }
        Ok(())
    }

    /// The media requests captured so far, as JSON.
    pub async fn media_capture_json(&self) -> XcelerateResult<String> {
        let entries = self.capture_entries.lock().await.clone();
        Ok(json!({ "count": entries.len(), "media": entries }).to_string())
    }

    /// Stops capturing and reassembles the captured segments into `path`.
    ///
    /// Video goes to `path`; when audio was captured too, it goes to
    /// `<name>.audio.m4a` beside it. Returns a human-readable summary.
    pub async fn save_capture(&self, path: String) -> XcelerateResult<String> {
        self.stop_media_capture().await?;
        let entries = self.capture_entries.lock().await.clone();
        let urls_of = |kind: &str| -> Vec<String> {
            entries
                .iter()
                .filter(|entry| entry["kind"] == json!(kind))
                .filter_map(|entry| entry["url"].as_str().map(str::to_string))
                .collect()
        };
        let video = urls_of("video");
        let audio = urls_of("audio");
        if video.is_empty() && audio.is_empty() {
            return Err(XcelerateError::NotFound(
                "no media requests were captured - the page may not have started playing".to_string(),
            ));
        }

        let mut reports = Vec::new();
        if video.is_empty() {
            // Audio-only stream.
            let bytes = self.write_urls(&audio, &path).await?;
            reports.push(format!("{path} ({bytes} bytes)"));
        } else {
            let bytes = self.write_urls(&video, &path).await?;
            reports.push(format!("{path} ({bytes} bytes)"));
            if !audio.is_empty() {
                let audio_path = super::media::split_audio_path(&path);
                let bytes = self.write_urls(&audio, &audio_path).await?;
                reports.push(format!("{audio_path} ({bytes} bytes)"));
            }
        }
        Ok(format!("saved {}", reports.join(" + ")))
    }

    /// Fetches each URL in order and concatenates the bytes into `path`.
    async fn write_urls(&self, urls: &[String], path: &str) -> XcelerateResult<u64> {
        super::media::create_parent(path).await?;
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(path)
            .await
            .map_err(super::media::io_error)?;
        let mut total: u64 = 0;
        for url in urls {
            let bytes = self.fetch_bytes(url).await?;
            file.write_all(&bytes)
                .await
                .map_err(super::media::io_error)?;
            total += bytes.len() as u64;
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::media_kind;

    #[test]
    fn classifies_media_responses() {
        assert_eq!(
            media_kind("video/mp4", "https://x/videoplayback?range=0-1"),
            Some("video")
        );
        assert_eq!(media_kind("audio/webm", "https://x/a"), Some("audio"));
        assert_eq!(
            media_kind("video/mp2t", "https://x/seg.ts"),
            Some("video")
        );
        assert_eq!(media_kind("text/html", "https://x/"), None);
        assert_eq!(media_kind("application/json", "https://x/api"), None);
    }
}
