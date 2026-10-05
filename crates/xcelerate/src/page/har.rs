//! HAR 1.2 network export (capability #5).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::CdpClient;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

/// Background pump that collects `Network.*` events into HAR entries.
///
/// Entries are assembled incrementally: `Network.requestWillBeSent` opens an
/// entry keyed by `requestId`, `Network.responseReceived` fills in the response,
/// and `Network.loadingFinished` / `Network.loadingFailed` finalize the entry
/// and append it to `entries`. Requests that never finish stay in the local
/// map and are dropped when the task is aborted.
async fn run_har(
    client: Arc<CdpClient>,
    session_id: String,
    entries: Arc<tokio::sync::Mutex<Vec<Value>>>,
) {
    let mut receiver = client.subscribe();
    let mut pending: HashMap<String, Value> = HashMap::new();
    loop {
        let value = match receiver.recv().await {
            Ok(value) => value,
            // A slow consumer only misses events; keep pumping.
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
                let entry = json!({
                    "requestId": request_id.clone(),
                    "timestamp": params.get("timestamp").cloned().unwrap_or(Value::Null),
                    "wallTime": params.get("wallTime").cloned().unwrap_or(Value::Null),
                    "request": params.get("request").cloned().unwrap_or(Value::Null),
                });
                pending.insert(request_id, entry);
            }
            "Network.responseReceived" => {
                if let Some(entry) = pending.get_mut(&request_id) {
                    entry["response"] = params.get("response").cloned().unwrap_or(Value::Null);
                    entry["responseTimestamp"] =
                        params.get("timestamp").cloned().unwrap_or(Value::Null);
                }
            }
            "Network.loadingFinished" => {
                if let Some(mut entry) = pending.remove(&request_id) {
                    entry["loadingFinished"] = params;
                    entries.lock().await.push(entry);
                }
            }
            "Network.loadingFailed" => {
                if let Some(mut entry) = pending.remove(&request_id) {
                    entry["loadingFailed"] = params;
                    entries.lock().await.push(entry);
                }
            }
            _ => {}
        }
    }
}

/// Reads a JSON number, tolerating integral values encoded as floats.
fn number(value: Option<&Value>) -> Option<f64> {
    value.and_then(Value::as_f64)
}

/// Converts CDP headers (an object or a `{name, value}` array) to HAR's
/// ordered `{name, value}` array form.
fn headers_to_har(headers: Option<&Value>) -> Value {
    let mut out = Vec::new();
    match headers {
        Some(Value::Object(map)) => {
            for (name, value) in map {
                let value = value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string());
                out.push(json!({ "name": name, "value": value }));
            }
        }
        Some(Value::Array(items)) => {
            for item in items {
                let name = item
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let value = item
                    .get("value")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        item.get("value").map(|v| v.to_string()).unwrap_or_default()
                    });
                out.push(json!({ "name": name, "value": value }));
            }
        }
        _ => {}
    }
    Value::Array(out)
}

/// Converts a count of days since the Unix epoch to a civil `(year, month, day)`
/// using Howard Hinnant's `civil_from_days` algorithm.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Formats seconds since the Unix epoch as an ISO 8601 UTC timestamp with
/// millisecond precision (`1970-01-01T00:00:00.000Z`).
fn format_iso8601(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0).round() as i64;
    let millis = total_ms.rem_euclid(1000);
    let secs = total_ms.div_euclid(1000);
    let epoch_days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(epoch_days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

/// Projects one internal record (request/response/finish) onto a HAR entry.
fn build_har_entry(record: &Value) -> Value {
    let method = record
        .pointer("/request/method")
        .and_then(Value::as_str)
        .unwrap_or("GET")
        .to_string();
    let url = record
        .pointer("/request/url")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let status = number(record.pointer("/response/status")).unwrap_or(0.0) as i64;
    let status_text = record
        .pointer("/response/statusText")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mime_type = record
        .pointer("/response/mimeType")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let content_size = number(record.pointer("/loadingFinished/encodedDataLength"))
        .or_else(|| number(record.pointer("/response/encodedDataLength")))
        .unwrap_or(0.0);

    let start_ts = number(record.get("timestamp"));
    let end_ts = number(record.pointer("/loadingFinished/timestamp"))
        .or_else(|| number(record.pointer("/loadingFailed/timestamp")))
        .or_else(|| number(record.get("responseTimestamp")));
    let time_ms = match (start_ts, end_ts) {
        (Some(start), Some(end)) if end > start => (end - start) * 1000.0,
        _ => 0.0,
    };

    let started_date_time = number(record.get("wallTime"))
        .map(format_iso8601)
        .unwrap_or_else(|| format_iso8601(0.0));

    json!({
        "startedDateTime": started_date_time,
        "time": time_ms,
        "request": {
            "method": method,
            "url": url,
            "httpVersion": "HTTP/1.1",
            "headers": headers_to_har(record.pointer("/request/headers")),
            "queryString": [],
            "headersSize": -1,
            "bodySize": -1,
        },
        "response": {
            "status": status,
            "statusText": status_text,
            "httpVersion": "HTTP/1.1",
            "headers": headers_to_har(record.pointer("/response/headers")),
            "content": {
                "size": content_size,
                "mimeType": mime_type,
            },
            "redirectURL": "",
            "headersSize": -1,
            "bodySize": -1,
        },
        "cache": {},
        "timings": { "send": 0, "wait": 0, "receive": 0 },
    })
}

impl Page {
    /// Starts recording network activity as HAR entries.
    ///
    /// Enables the CDP `Network` domain, clears any previously collected
    /// entries, and spawns a background task that fills [`Page::har_entries`]
    /// from `Network.*` events. If a recording task is already running it is
    /// aborted first, so calling this twice restarts collection.
    pub async fn start_har_recording(&self) -> XcelerateResult<()> {
        if let Some(handle) = self.har_task.lock().await.take() {
            handle.abort();
        }
        self.har_entries.lock().await.clear();
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Network.enable",
                serde_json::json!({}),
            )
            .await?;
        let handle = tokio::spawn(run_har(
            Arc::clone(&self.client),
            self.session_id.clone(),
            Arc::clone(&self.har_entries),
        ));
        *self.har_task.lock().await = Some(handle);
        Ok(())
    }

    /// Stops recording network activity.
    ///
    /// Aborts the background task and clears the stored handle, but keeps the
    /// entries collected so far so [`Page::save_har`] can still write them.
    pub async fn stop_har_recording(&self) -> XcelerateResult<()> {
        if let Some(handle) = self.har_task.lock().await.take() {
            handle.abort();
        }
        Ok(())
    }

    /// Stops recording and writes the collected entries to `path` as HAR 1.2.
    ///
    /// Returns `path` on success.
    pub async fn save_har(&self, path: String) -> XcelerateResult<String> {
        self.stop_har_recording().await?;
        let entries = self.har_entries.lock().await.clone();
        let har = Self::build_har(&entries);
        let content = serde_json::to_string_pretty(&har)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        tokio::fs::write(&path, content)
            .await
            .map_err(|e| XcelerateError::NotFound(format!("failed to write HAR: {e}")))?;
        Ok(path)
    }

    /// Builds a HAR 1.2 document from internally recorded entries.
    ///
    /// Pure and synchronous: each input value is a record holding the
    /// `requestWillBeSent` request, an optional `responseReceived` response,
    /// and the `loadingFinished` / `loadingFailed` event that finalized it.
    pub(crate) fn build_har(entries: &[serde_json::Value]) -> serde_json::Value {
        let har_entries: Vec<Value> = entries.iter().map(build_har_entry).collect();
        json!({
            "log": {
                "version": "1.2",
                "creator": {
                    "name": "xcelerate",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "entries": har_entries,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Page;
    use serde_json::{Value, json};

    /// Mirrors the record shape assembled by `run_har` for one request.
    fn record(request: &Value, response: &Value, finished: &Value) -> Value {
        json!({
            "requestId": "1",
            "timestamp": request.get("timestamp").cloned().unwrap_or(Value::Null),
            "wallTime": request.get("wallTime").cloned().unwrap_or(Value::Null),
            "request": request.get("request").cloned().unwrap_or(Value::Null),
            "response": response.get("response").cloned().unwrap_or(Value::Null),
            "responseTimestamp": response.get("timestamp").cloned().unwrap_or(Value::Null),
            "loadingFinished": finished,
        })
    }

    fn request_will_be_sent() -> Value {
        json!({
            "method": "Network.requestWillBeSent",
            "params": {
                "requestId": "1",
                "timestamp": 100.0,
                "wallTime": 1_600_000_000.0,
                "request": {
                    "url": "https://example.com/api",
                    "method": "GET",
                    "headers": { "Accept": "*/*" },
                },
            },
        })
    }

    fn response_received() -> Value {
        json!({
            "method": "Network.responseReceived",
            "params": {
                "requestId": "1",
                "timestamp": 100.25,
                "response": {
                    "status": 200,
                    "statusText": "OK",
                    "mimeType": "application/json",
                    "headers": { "Content-Type": "application/json" },
                },
            },
        })
    }

    fn loading_finished() -> Value {
        json!({
            "method": "Network.loadingFinished",
            "params": {
                "requestId": "1",
                "timestamp": 100.5,
                "encodedDataLength": 42,
            },
        })
    }

    #[test]
    fn build_har_produces_one_entry_with_url_and_status() {
        let request = request_will_be_sent();
        let response = response_received();
        let finished = loading_finished();
        let entry = record(
            request.get("params").unwrap(),
            response.get("params").unwrap(),
            finished.get("params").unwrap(),
        );

        let har = Page::build_har(&[entry]);

        assert_eq!(
            har.pointer("/log/version").and_then(Value::as_str),
            Some("1.2")
        );
        assert_eq!(
            har.pointer("/log/creator/name").and_then(Value::as_str),
            Some("xcelerate")
        );

        let entries = har
            .pointer("/log/entries")
            .and_then(Value::as_array)
            .expect("entries array");
        assert_eq!(entries.len(), 1);

        let entry = &entries[0];
        assert_eq!(
            entry.pointer("/request/url").and_then(Value::as_str),
            Some("https://example.com/api")
        );
        assert_eq!(
            entry.pointer("/response/status").and_then(Value::as_i64),
            Some(200)
        );
        assert_eq!(
            entry
                .pointer("/response/content/mimeType")
                .and_then(Value::as_str),
            Some("application/json")
        );
        // 100.5s - 100.0s, in milliseconds.
        assert_eq!(entry.get("time").and_then(Value::as_f64), Some(500.0));
    }

    #[test]
    fn build_har_encodes_request_headers_as_name_value_array() {
        let record = record(
            request_will_be_sent().get("params").unwrap(),
            response_received().get("params").unwrap(),
            loading_finished().get("params").unwrap(),
        );
        let har = Page::build_har(&[record]);
        let headers = har
            .pointer("/log/entries/0/request/headers")
            .and_then(Value::as_array)
            .expect("headers array");
        assert_eq!(headers.len(), 1);
        assert_eq!(
            headers[0].get("name").and_then(Value::as_str),
            Some("Accept")
        );
        assert_eq!(headers[0].get("value").and_then(Value::as_str), Some("*/*"));
    }

    #[test]
    fn build_har_handles_empty_entries() {
        let har = Page::build_har(&[]);
        assert_eq!(
            har.pointer("/log/version").and_then(Value::as_str),
            Some("1.2")
        );
        assert_eq!(
            har.pointer("/log/entries")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn iso8601_formats_the_epoch() {
        assert_eq!(super::format_iso8601(0.0), "1970-01-01T00:00:00.000Z");
    }
}
