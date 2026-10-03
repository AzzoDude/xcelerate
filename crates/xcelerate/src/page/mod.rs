use crate::CdpClient;
use crate::element::Element;
use crate::error::{XcelerateError, XcelerateResult};
use browser_protocol::emulation::{
    ClearDeviceMetricsOverrideParams, MediaFeature, SetDeviceMetricsOverrideParams,
    SetEmulatedMediaParams, SetScriptExecutionDisabledParams, SetUserAgentOverrideParams,
};
use browser_protocol::network::{
    EmulateNetworkConditionsParams, GetCookiesParams, SetCacheDisabledParams,
    SetExtraHTTPHeadersParams,
};
use browser_protocol::page::{
    BringToFrontParams, CaptureScreenshotParams, CloseParams, EnableParams, GetLayoutMetricsParams,
    NavigateParams, ReloadParams,
};
use browser_protocol::performance::GetMetricsParams;
use std::sync::Arc;

mod intercept;
mod rng;

use intercept::run_interception;
pub(crate) use rng::Lcg;

#[derive(uniffi::Object)]
pub struct Page {
    pub(crate) client: Arc<CdpClient>,
    pub(crate) session_id: String,
    pub(crate) target_id: String,
    pub(crate) mouse_x: std::sync::Mutex<f64>,
    pub(crate) mouse_y: std::sync::Mutex<f64>,
    pub(crate) events: tokio::sync::Mutex<Vec<String>>,
    pub(crate) routes: Arc<tokio::sync::Mutex<Vec<RouteRule>>>,
    pub(crate) requests: Arc<tokio::sync::Mutex<Vec<serde_json::Value>>>,
    pub(crate) interception_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    pub(crate) credentials: Arc<tokio::sync::Mutex<Option<(String, String)>>>,
    pub(crate) drag_interception: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) default_timeout_ms: std::sync::atomic::AtomicU64,
}

/// A declarative network-interception rule.
#[derive(Clone)]
pub(crate) struct RouteRule {
    pattern: String,
    /// One of `continue`, `abort`, `fulfill`.
    action: String,
    body: Option<String>,
    content_type: Option<String>,
}

#[uniffi::export(async_runtime = "tokio")]
impl Page {
    /// Finds an element matching the CSS selector.
    pub async fn find_element(self: Arc<Self>, selector: String) -> XcelerateResult<Arc<Element>> {
        // JSON-escape the selector so quotes/backslashes cannot break out of the
        // JS string literal (and to avoid injection into the expression).
        let quoted = serde_json::to_string(&selector)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let js = format!("document.querySelector({quoted})");

        // Evaluate returns complex JSON, we handle it internally
        self.client
            .execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: js.into(),
                    ..Default::default()
                },
            )
            .await
            .map_err(XcelerateError::from)
            .and_then(|result| {
                if let Some(obj_id) = result.result.object_id {
                    Ok(Arc::new(Element {
                        page: self.clone(),
                        object_id: obj_id.into_owned(),
                    }))
                } else {
                    Err(XcelerateError::NotFound(selector))
                }
            })
    }

    /// Waits for an element matching the selector to appear in the DOM.
    pub async fn wait_for_selector(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Arc<Element>> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(self.default_timeout());

        while start.elapsed() < timeout {
            if let Ok(element) = self.clone().find_element(selector.clone()).await {
                return Ok(element);
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }

        Err(XcelerateError::NotFound(format!(
            "Timeout waiting for selector: {}",
            selector
        )))
    }

    /// Waits for the page to finish loading.
    pub async fn wait_for_navigation(&self) -> XcelerateResult<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(self.default_timeout());

        while start.elapsed() < timeout {
            // Internal call to evaluate
            let res = self
                .client
                .execute_with_session(
                    Some(&self.session_id),
                    js_protocol::runtime::EvaluateParams {
                        expression: "document.readyState".into(),
                        ..Default::default()
                    },
                )
                .await?;

            if res
                .result
                .value
                .is_some_and(|v| v.as_str() == Some("complete"))
            {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }

        Err(XcelerateError::NotFound("Navigation timeout".into()))
    }

    /// Reloads the page.
    pub async fn reload(&self) -> XcelerateResult<()> {
        self.client
            .execute_with_session(
                Some(&self.session_id),
                ReloadParams {
                    ..Default::default()
                },
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Navigates to a URL.
    pub async fn navigate(&self, url: String) -> XcelerateResult<()> {
        self.client
            .execute_with_session(
                Some(&self.session_id),
                NavigateParams {
                    url: url.into(),
                    ..Default::default()
                },
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Returns the page title.
    pub async fn title(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: "document.title".into(),
                    ..Default::default()
                },
            )
            .await?;
        Ok(res
            .result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default())
    }

    /// Returns the full HTML content of the page.
    pub async fn content(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: "document.documentElement.outerHTML".into(),
                    ..Default::default()
                },
            )
            .await?;
        Ok(res
            .result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default())
    }

    pub async fn screenshot(&self) -> XcelerateResult<Vec<u8>> {
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                CaptureScreenshotParams {
                    ..Default::default()
                },
            )
            .await?;
        self.decode_base64(res.data.into_owned())
    }

    pub async fn screenshot_full(&self) -> XcelerateResult<Vec<u8>> {
        let _ = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                EnableParams {
                    ..Default::default()
                },
            )
            .await?;

        let metrics = self
            .client
            .execute_with_session(Some(&self.session_id), GetLayoutMetricsParams {})
            .await?;

        let width = metrics.content_size.width as u64;
        let height = metrics.content_size.height as i64;

        let mut params = SetDeviceMetricsOverrideParams {
            ..Default::default()
        };
        params.width = width;
        params.height = height;
        params.device_scale_factor = 1.0;
        params.mobile = false;

        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await?;

        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                CaptureScreenshotParams {
                    ..Default::default()
                },
            )
            .await?;

        let _ = self
            .client
            .execute_with_session(Some(&self.session_id), ClearDeviceMetricsOverrideParams {})
            .await?;

        self.decode_base64(res.data.into_owned())
    }

    pub async fn pdf(&self) -> XcelerateResult<Vec<u8>> {
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                browser_protocol::page::PrintToPDFParams {
                    ..Default::default()
                },
            )
            .await?;
        self.decode_base64(res.data.into_owned())
    }

    /// Evaluates a script on every new document.
    pub async fn add_script_to_evaluate_on_new_document(
        &self,
        source: String,
    ) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams {
                    source: source.into(),
                    ..Default::default()
                },
            )
            .await?;
        Ok(res.identifier.into_owned())
    }

    pub async fn go_back(&self) -> XcelerateResult<()> {
        let _ = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: "window.history.back()".into(),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }

    fn decode_base64(&self, data: String) -> XcelerateResult<Vec<u8>> {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD
            .decode(data)
            .map_err(|e| XcelerateError::SerdeError(format!("Base64 decode failed: {}", e)))
    }

    /// Moves the mouse cursor from the current position to the target (x, y) along a realistic Bezier curve.
    pub async fn move_mouse(self: Arc<Self>, x: f64, y: f64) -> XcelerateResult<Arc<Self>> {
        let (start_x, start_y) = {
            let cx = *self.mouse_x.lock().unwrap();
            let cy = *self.mouse_y.lock().unwrap();
            (cx, cy)
        };

        let dx = x - start_x;
        let dy = y - start_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 1.0 {
            {
                let mut cx = self.mouse_x.lock().unwrap();
                let mut cy = self.mouse_y.lock().unwrap();
                *cx = x;
                *cy = y;
            }
            return Ok(self);
        }

        let mut rng = Lcg::new();
        let (px, py) = if distance > 0.0 {
            (-dy / distance, dx / distance)
        } else {
            (0.0, 0.0)
        };

        let offset_scale1 = rng.range(-0.2, 0.2) * distance;
        let offset_scale2 = rng.range(-0.2, 0.2) * distance;

        let p1_x = start_x + dx * 0.25 + px * offset_scale1;
        let p1_y = start_y + dy * 0.25 + py * offset_scale1;

        let p2_x = start_x + dx * 0.75 + px * offset_scale2;
        let p2_y = start_y + dy * 0.75 + py * offset_scale2;

        let steps_f = (distance / rng.range(12.0, 25.0)).clamp(12.0, 60.0);
        let steps = steps_f as usize;

        for i in 1..=steps {
            let s = (i as f64) / (steps as f64);

            let t = if s < 0.5 {
                4.0 * s * s * s
            } else {
                let f = -2.0 * s + 2.0;
                1.0 - f * f * f / 2.0
            };

            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            let t2 = t * t;
            let t3 = t2 * t;

            let mut curr_x = mt3 * start_x + 3.0 * mt2 * t * p1_x + 3.0 * mt * t2 * p2_x + t3 * x;
            let mut curr_y = mt3 * start_y + 3.0 * mt2 * t * p1_y + 3.0 * mt * t2 * p2_y + t3 * y;

            if i < steps {
                curr_x += rng.range(-0.4, 0.4);
                curr_y += rng.range(-0.4, 0.4);
            }

            let params = browser_protocol::input::DispatchMouseEventParams {
                type_: "mouseMoved".into(),
                x: curr_x,
                y: curr_y,
                ..Default::default()
            };

            self.client
                .execute_with_session(Some(&self.session_id), params)
                .await?;

            let delay_ms = rng.range(6.0, 14.0) as u64;
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }

        {
            let mut cx = self.mouse_x.lock().unwrap();
            let mut cy = self.mouse_y.lock().unwrap();
            *cx = x;
            *cy = y;
        }

        Ok(self)
    }

    /// Triggers a mousePress event at the current mouse coordinates.
    pub async fn mouse_down(self: Arc<Self>, button: String) -> XcelerateResult<Arc<Self>> {
        let (cx, cy) = {
            let x = *self.mouse_x.lock().unwrap();
            let y = *self.mouse_y.lock().unwrap();
            (x, y)
        };
        let btn = match button.as_str() {
            "left" => browser_protocol::input::MouseButton::Left,
            "right" => browser_protocol::input::MouseButton::Right,
            "middle" => browser_protocol::input::MouseButton::Middle,
            "back" => browser_protocol::input::MouseButton::Back,
            "forward" => browser_protocol::input::MouseButton::Forward,
            _ => browser_protocol::input::MouseButton::None,
        };
        let params = browser_protocol::input::DispatchMouseEventParams {
            type_: "mousePressed".into(),
            x: cx,
            y: cy,
            button: Some(btn),
            click_count: Some(1),
            ..Default::default()
        };
        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await?;
        Ok(self)
    }

    /// Triggers a mouseReleased event at the current mouse coordinates.
    pub async fn mouse_up(self: Arc<Self>, button: String) -> XcelerateResult<Arc<Self>> {
        let (cx, cy) = {
            let x = *self.mouse_x.lock().unwrap();
            let y = *self.mouse_y.lock().unwrap();
            (x, y)
        };
        let btn = match button.as_str() {
            "left" => browser_protocol::input::MouseButton::Left,
            "right" => browser_protocol::input::MouseButton::Right,
            "middle" => browser_protocol::input::MouseButton::Middle,
            "back" => browser_protocol::input::MouseButton::Back,
            "forward" => browser_protocol::input::MouseButton::Forward,
            _ => browser_protocol::input::MouseButton::None,
        };
        let params = browser_protocol::input::DispatchMouseEventParams {
            type_: "mouseReleased".into(),
            x: cx,
            y: cy,
            button: Some(btn),
            click_count: Some(1),
            ..Default::default()
        };
        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await?;
        Ok(self)
    }

    /// Moves the mouse to (x, y) and performs a click (down & up) with human-like delays.
    pub async fn click_mouse(self: Arc<Self>, x: f64, y: f64) -> XcelerateResult<Arc<Self>> {
        let page = self.clone();
        page.clone().move_mouse(x, y).await?;

        let mut rng = Lcg::new();
        let latency_delay = rng.range(50.0, 130.0) as u64;
        tokio::time::sleep(tokio::time::Duration::from_millis(latency_delay)).await;

        page.clone().mouse_down("left".to_string()).await?;

        let hold_delay = rng.range(60.0, 140.0) as u64;
        tokio::time::sleep(tokio::time::Duration::from_millis(hold_delay)).await;

        page.clone().mouse_up("left".to_string()).await?;

        Ok(self)
    }

    // -----------------------------------------------------------------------
    // Generic JS / CDP bridge
    //
    // These primitives let the API-style adapters express the broad upstream
    // surface (Playwright / Puppeteer / Selenium) without a hand-written core
    // method per library member. Values cross the boundary as JSON strings so
    // the whole facade stays UniFFI-compatible.
    // -----------------------------------------------------------------------

    /// Evaluates JavaScript in the page and returns the result as a JSON string.
    pub async fn evaluate_json(&self, expression: String) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: expression.into(),
                    return_by_value: Some(true),
                    await_promise: Some(true),
                    ..Default::default()
                },
            )
            .await?;
        Ok(res
            .result
            .value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string()))
    }

    /// Evaluates JavaScript and coerces the result to a string.
    pub async fn evaluate_string(&self, expression: String) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: expression.into(),
                    return_by_value: Some(true),
                    await_promise: Some(true),
                    ..Default::default()
                },
            )
            .await?;
        Ok(res
            .result
            .value
            .and_then(|v| match v {
                serde_json::Value::String(s) => Some(s),
                serde_json::Value::Null => None,
                other => Some(other.to_string()),
            })
            .unwrap_or_default())
    }

    /// Evaluates JavaScript and coerces the result to a bool.
    pub async fn evaluate_bool(&self, expression: String) -> XcelerateResult<bool> {
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: expression.into(),
                    return_by_value: Some(true),
                    await_promise: Some(true),
                    ..Default::default()
                },
            )
            .await?;
        Ok(res.result.value.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// Evaluates JavaScript and returns the resulting object as an [`Element`].
    pub async fn evaluate_handle(
        self: Arc<Self>,
        expression: String,
    ) -> XcelerateResult<Arc<Element>> {
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: expression.clone().into(),
                    return_by_value: Some(false),
                    await_promise: Some(true),
                    ..Default::default()
                },
            )
            .await?;
        if let Some(object_id) = res.result.object_id {
            Ok(Arc::new(Element {
                page: self.clone(),
                object_id: object_id.into_owned(),
            }))
        } else {
            Err(XcelerateError::NotFound(expression))
        }
    }

    /// Returns every element matching the CSS selector.
    ///
    /// Uses two round trips (fetch the node list, then read its properties)
    /// instead of one `evaluate` per match.
    pub async fn query_selector_all(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Vec<Arc<Element>>> {
        let quoted = serde_json::to_string(&selector)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let res = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                js_protocol::runtime::EvaluateParams {
                    expression: format!("Array.from(document.querySelectorAll({quoted}))").into(),
                    return_by_value: Some(false),
                    ..Default::default()
                },
            )
            .await?;
        match res.result.object_id {
            Some(object_id) => {
                collect_elements(&self.client, &self.session_id, &self, object_id).await
            }
            None => Ok(Vec::new()),
        }
    }

    /// Returns the current document URL.
    pub async fn url(&self) -> XcelerateResult<String> {
        self.evaluate_string("location.href".to_string()).await
    }

    /// Replaces the document content.
    pub async fn set_content(&self, html: String) -> XcelerateResult<()> {
        let quoted =
            serde_json::to_string(&html).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_json(format!(
            "document.open();document.write({quoted});document.close();"
        ))
        .await?;
        Ok(())
    }

    /// Navigates forward in history.
    pub async fn go_forward(&self) -> XcelerateResult<()> {
        self.evaluate_json("history.forward()".to_string()).await?;
        Ok(())
    }

    /// Closes the page.
    pub async fn close(&self) -> XcelerateResult<()> {
        self.client
            .execute_with_session(Some(&self.session_id), CloseParams {})
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Brings the page to the front.
    pub async fn bring_to_front(&self) -> XcelerateResult<()> {
        self.client
            .execute_with_session(Some(&self.session_id), BringToFrontParams {})
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Overrides the viewport size.
    pub async fn set_viewport_size(&self, width: u64, height: i64) -> XcelerateResult<()> {
        let params = SetDeviceMetricsOverrideParams {
            width,
            height,
            device_scale_factor: 1.0,
            mobile: false,
            ..Default::default()
        };
        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Emulates a media type and/or colour scheme.
    pub async fn emulate_media(
        &self,
        media: Option<String>,
        color_scheme: Option<String>,
    ) -> XcelerateResult<()> {
        let features = color_scheme.map(|scheme| {
            vec![MediaFeature {
                name: "prefers-color-scheme".into(),
                value: scheme.into(),
            }]
        });
        let params = SetEmulatedMediaParams {
            media: media.map(Into::into),
            features,
        };
        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Sets extra HTTP headers for every request from this page.
    pub async fn set_extra_http_headers(&self, headers_json: String) -> XcelerateResult<()> {
        let headers: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&headers_json)
                .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.client
            .execute_with_session(
                Some(&self.session_id),
                SetExtraHTTPHeadersParams { headers },
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Injects a `<style>` element and returns the injected content.
    pub async fn add_style_tag(&self, content: String) -> XcelerateResult<String> {
        let quoted = serde_json::to_string(&content)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_string(format!(
            "(function(){{const s=document.createElement('style');s.textContent={quoted};document.head.appendChild(s);return s.textContent;}})()"
        ))
        .await
    }

    /// Polls `expression` until it evaluates truthy or `timeout_ms` elapses.
    pub async fn wait_for_function(
        &self,
        expression: String,
        timeout_ms: u64,
    ) -> XcelerateResult<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms.max(1));
        loop {
            if self
                .evaluate_bool(expression.clone())
                .await
                .unwrap_or(false)
            {
                return Ok(());
            }
            if start.elapsed() >= timeout {
                return Err(XcelerateError::NotFound(format!(
                    "Timeout waiting for function: {expression}"
                )));
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    /// Focuses the element matching `selector` and presses `key`.
    pub async fn press(self: Arc<Self>, selector: String, key: String) -> XcelerateResult<()> {
        let element = self.clone().find_element(selector).await?;
        element.press(key).await?;
        Ok(())
    }

    /// Selects options by value/label on the matching `<select>`.
    pub async fn select_option(
        self: Arc<Self>,
        selector: String,
        values_json: String,
    ) -> XcelerateResult<()> {
        let element = self.clone().find_element(selector).await?;
        element.select_option(values_json).await?;
        Ok(())
    }

    /// Sets the files of the matching `<input type="file">`.
    pub async fn set_input_files(
        self: Arc<Self>,
        selector: String,
        files_json: String,
    ) -> XcelerateResult<()> {
        let element = self.clone().find_element(selector).await?;
        element.set_input_files(files_json).await?;
        Ok(())
    }

    /// Calls a JS function with JSON-encoded arguments, returning JSON text.
    ///
    /// This is the shim the adapters use to express the broad upstream surface
    /// as data (a function body per method) rather than a core method per member.
    pub async fn call_json(&self, function: String, args_json: String) -> XcelerateResult<String> {
        let values: Vec<serde_json::Value> = serde_json::from_str(&args_json).unwrap_or_default();
        let args = values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.evaluate_json(format!("({function})({args})")).await
    }

    /// Like [`Page::call_json`] but coerces the result to a string.
    pub async fn call_string(
        &self,
        function: String,
        args_json: String,
    ) -> XcelerateResult<String> {
        let json = self.call_json(function, args_json).await?;
        Ok(match serde_json::from_str::<serde_json::Value>(&json) {
            Ok(serde_json::Value::String(s)) => s,
            Ok(serde_json::Value::Null) => String::new(),
            Ok(other) => other.to_string(),
            Err(_) => json,
        })
    }

    /// Like [`Page::call_json`] but coerces the result to a bool.
    pub async fn call_bool(&self, function: String, args_json: String) -> XcelerateResult<bool> {
        let json = self.call_json(function, args_json).await?;
        Ok(serde_json::from_str::<serde_json::Value>(&json)
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    /// Returns the first node matching an XPath expression as an [`Element`].
    pub async fn query_selector_xpath(
        self: Arc<Self>,
        xpath: String,
    ) -> XcelerateResult<Arc<Element>> {
        let quoted =
            serde_json::to_string(&xpath).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_handle(format!(
            "document.evaluate({quoted}, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue"
        ))
        .await
    }

    /// Runs a JS function against the element matching `selector` (`$eval`).
    pub async fn call_on_selector(
        &self,
        selector: String,
        expression: String,
    ) -> XcelerateResult<String> {
        let sel = serde_json::to_string(&selector)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let expr = serde_json::to_string(&expression)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_json(format!(
            "(function(){{const el=document.querySelector({sel});if(!el)return null;return (new Function('el','return ('+{expr}+')(el);'))(el);}})()"
        ))
        .await
    }

    /// Runs a JS function against every element matching `selector` (`$$eval`).
    pub async fn call_on_selector_all(
        &self,
        selector: String,
        expression: String,
    ) -> XcelerateResult<String> {
        let sel = serde_json::to_string(&selector)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let expr = serde_json::to_string(&expression)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_json(format!(
            "(function(){{const els=Array.from(document.querySelectorAll({sel}));const fn=new Function('el','return ('+{expr}+')(el);');return els.map(fn);}})()"
        ))
        .await
    }

    /// Overrides `navigator.userAgent` for this page.
    pub async fn set_user_agent(
        &self,
        user_agent: String,
        accept_language: Option<String>,
    ) -> XcelerateResult<()> {
        let params = SetUserAgentOverrideParams {
            user_agent: user_agent.into(),
            accept_language: accept_language.map(Into::into),
            ..Default::default()
        };
        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Enables or disables the HTTP cache.
    pub async fn set_cache_enabled(&self, enabled: bool) -> XcelerateResult<()> {
        let params = SetCacheDisabledParams {
            cache_disabled: !enabled,
        };
        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Enables or disables JavaScript execution.
    pub async fn set_javascript_enabled(&self, enabled: bool) -> XcelerateResult<()> {
        let params = SetScriptExecutionDisabledParams { value: !enabled };
        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Toggles offline mode.
    pub async fn set_offline(&self, offline: bool) -> XcelerateResult<()> {
        let params = EmulateNetworkConditionsParams {
            offline,
            latency: 0.0,
            download_throughput: -1.0,
            upload_throughput: -1.0,
            ..Default::default()
        };
        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Returns the cookies visible to this page as a JSON array.
    pub async fn cookies(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_with_session(Some(&self.session_id), GetCookiesParams { urls: None })
            .await?;
        Ok(serde_json::to_string(&res.cookies).unwrap_or_else(|_| "[]".to_string()))
    }

    /// Returns a single cookie by name as JSON (or null).
    pub async fn cookie(&self, name: String) -> XcelerateResult<String> {
        let cookies: serde_json::Value =
            serde_json::from_str(&self.cookies().await?).unwrap_or(serde_json::json!([]));
        let found = cookies
            .as_array()
            .and_then(|list| {
                list.iter().find(|cookie| {
                    cookie.get("name").and_then(|value| value.as_str()) == Some(name.as_str())
                })
            })
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        Ok(found.to_string())
    }

    /// Returns the page performance metrics as a JSON object.
    pub async fn metrics(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_with_session(Some(&self.session_id), GetMetricsParams {})
            .await?;
        let map: serde_json::Map<String, serde_json::Value> = res
            .metrics
            .into_iter()
            .map(|metric| (metric.name.into_owned(), serde_json::json!(metric.value)))
            .collect();
        Ok(serde_json::Value::Object(map).to_string())
    }

    /// Escape hatch: sends an arbitrary CDP command and returns its JSON result.
    ///
    /// The adapters use this to express CDP-backed library methods as data
    /// (a method name + parameter template), keeping the core surface small.
    pub async fn execute_cdp_cmd(
        &self,
        method: String,
        params_json: String,
    ) -> XcelerateResult<String> {
        let params: serde_json::Value =
            serde_json::from_str(&params_json).unwrap_or(serde_json::Value::Null);
        let res = self
            .client
            .execute_raw_with_session(Some(&self.session_id), &method, params)
            .await?;
        Ok(res.to_string())
    }

    /// Finds an element whose text content contains `text`.
    pub async fn get_by_text(self: Arc<Self>, text: String) -> XcelerateResult<Arc<Element>> {
        let quoted =
            serde_json::to_string(&text).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_handle(format!(
            "Array.from(document.querySelectorAll('*')).find(function(e){{return e.textContent.includes({quoted});}})"
        ))
        .await
    }

    /// Finds an element by ARIA role (falls back to a tag-name lookup).
    pub async fn get_by_role(self: Arc<Self>, role: String) -> XcelerateResult<Arc<Element>> {
        let quoted =
            serde_json::to_string(&role).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_handle(format!(
            "document.querySelector('[role=\"'+{quoted}+'\"]')||document.querySelector({quoted})"
        ))
        .await
    }

    /// Finds a form control by its associated `<label>` text.
    pub async fn get_by_label(self: Arc<Self>, label: String) -> XcelerateResult<Arc<Element>> {
        let quoted =
            serde_json::to_string(&label).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_handle(format!(
            "(function(l){{const labels=Array.from(document.querySelectorAll('label')).filter(function(x){{return x.textContent.trim().includes(l);}});if(!labels.length)return null;const lab=labels[0];if(lab.control)return lab.control;const f=lab.getAttribute('for');return f?document.getElementById(f):null;}})({quoted})"
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Events
    //
    // A real subscription registry plus a `wait_for_event` that consumes the
    // CDP event broadcast, so the adapters' event/expectation surface works.
    // -----------------------------------------------------------------------

    /// Registers interest in a CDP event name.
    pub async fn on(&self, event_name: String) {
        let mut events = self.events.lock().await;
        if !events.contains(&event_name) {
            events.push(event_name);
        }
    }

    /// Alias for [`Page::on`].
    pub async fn once(&self, event_name: String) {
        self.on(event_name).await;
    }

    /// Removes a single registered event listener.
    pub async fn remove_listener(&self, event_name: String) {
        let mut events = self.events.lock().await;
        events.retain(|name| name != &event_name);
    }

    /// Removes every registered event listener.
    pub async fn remove_all_listeners(&self) {
        self.events.lock().await.clear();
    }

    /// Returns the registered event names.
    pub async fn event_names(&self) -> Vec<String> {
        self.events.lock().await.clone()
    }

    /// Whether an event name is registered.
    pub async fn listens_to(&self, event_name: String) -> bool {
        self.events.lock().await.contains(&event_name)
    }

    /// Waits for the next CDP event named `event_name` and returns its params.
    ///
    /// The relevant domain is enabled first (best effort), so callers do not
    /// have to.
    pub async fn wait_for_event(
        &self,
        event_name: String,
        timeout_ms: u64,
    ) -> XcelerateResult<String> {
        let domain = event_name.split('.').next().unwrap_or("");
        let enable: Option<(&str, serde_json::Value)> = match domain {
            "Network" => Some(("Network.enable", serde_json::json!({}))),
            "Page" => Some(("Page.enable", serde_json::json!({}))),
            "Runtime" => Some(("Runtime.enable", serde_json::json!({}))),
            "Log" => Some(("Log.enable", serde_json::json!({}))),
            "Target" => Some((
                "Target.setDiscoverTargets",
                serde_json::json!({ "discover": true }),
            )),
            "DOM" => Some(("DOM.enable", serde_json::json!({}))),
            "DeviceAccess" => Some(("DeviceAccess.enable", serde_json::json!({}))),
            "Fetch" => Some(("Fetch.enable", serde_json::json!({}))),
            _ => None,
        };
        if let Some((method, params)) = enable {
            let _ = self
                .client
                .execute_raw_with_session(Some(&self.session_id), method, params)
                .await;
        }

        let mut receiver = self.client.subscribe();
        let timeout = std::time::Duration::from_millis(timeout_ms.max(1));
        let start = std::time::Instant::now();
        loop {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Err(XcelerateError::NotFound(format!(
                    "Timeout waiting for event: {event_name}"
                )));
            }
            match tokio::time::timeout(remaining, receiver.recv()).await {
                Ok(Ok(value)) => {
                    if value.get("method").and_then(|m| m.as_str()) == Some(event_name.as_str()) {
                        return Ok(value
                            .get("params")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null)
                            .to_string());
                    }
                }
                Ok(Err(_)) => continue,
                Err(_) => {
                    return Err(XcelerateError::NotFound(format!(
                        "Timeout waiting for event: {event_name}"
                    )));
                }
            }
        }
    }

    /// [`Page::wait_for_event`] with the page's default timeout.
    pub async fn wait_for_event_default(&self, event_name: String) -> XcelerateResult<String> {
        self.wait_for_event(event_name, self.default_timeout())
            .await
    }

    // -----------------------------------------------------------------------
    // Window geometry (Selenium WebDriver)
    // -----------------------------------------------------------------------

    async fn window_id(&self) -> XcelerateResult<i64> {
        let res = self
            .client
            .execute_raw(
                "Browser.getWindowForTarget",
                serde_json::json!({ "targetId": self.target_id }),
            )
            .await?;
        Ok(res.get("windowId").and_then(|v| v.as_i64()).unwrap_or(0))
    }

    async fn raw_window_bounds(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw(
                "Browser.getWindowForTarget",
                serde_json::json!({ "targetId": self.target_id }),
            )
            .await?;
        Ok(res
            .get("bounds")
            .cloned()
            .unwrap_or(serde_json::Value::Null)
            .to_string())
    }

    /// Returns the window bounds as JSON (`{left,top,width,height,windowState}`).
    pub async fn window_rect(&self) -> XcelerateResult<String> {
        self.raw_window_bounds().await
    }

    /// Returns the window size as JSON (`{width,height}`).
    pub async fn window_size(&self) -> XcelerateResult<String> {
        let bounds: serde_json::Value =
            serde_json::from_str(&self.raw_window_bounds().await?).unwrap_or_default();
        Ok(serde_json::json!({
            "width": bounds.get("width"),
            "height": bounds.get("height")
        })
        .to_string())
    }

    /// Returns the window position as JSON (`{x,y}`).
    pub async fn window_position(&self) -> XcelerateResult<String> {
        let bounds: serde_json::Value =
            serde_json::from_str(&self.raw_window_bounds().await?).unwrap_or_default();
        Ok(serde_json::json!({
            "x": bounds.get("left"),
            "y": bounds.get("top")
        })
        .to_string())
    }

    /// Moves and resizes the window.
    pub async fn set_window_bounds(
        &self,
        left: i64,
        top: i64,
        width: i64,
        height: i64,
    ) -> XcelerateResult<()> {
        let window_id = self.window_id().await?;
        self.client
            .execute_raw(
                "Browser.setWindowBounds",
                serde_json::json!({
                    "windowId": window_id,
                    "bounds": { "left": left, "top": top, "width": width, "height": height }
                }),
            )
            .await?;
        Ok(())
    }

    /// Sets the window state (`normal` | `minimized` | `maximized` | `fullscreen`).
    pub async fn set_window_state(&self, state: String) -> XcelerateResult<()> {
        let window_id = self.window_id().await?;
        self.client
            .execute_raw(
                "Browser.setWindowBounds",
                serde_json::json!({
                    "windowId": window_id,
                    "bounds": { "windowState": state }
                }),
            )
            .await?;
        Ok(())
    }

    /// Resizes the window, preserving its position.
    pub async fn set_window_size(&self, width: i64, height: i64) -> XcelerateResult<()> {
        let bounds: serde_json::Value =
            serde_json::from_str(&self.raw_window_bounds().await?).unwrap_or_default();
        let left = bounds.get("left").and_then(|v| v.as_i64()).unwrap_or(0);
        let top = bounds.get("top").and_then(|v| v.as_i64()).unwrap_or(0);
        self.set_window_bounds(left, top, width, height).await
    }

    /// Moves the window, preserving its size.
    pub async fn set_window_position(&self, x: i64, y: i64) -> XcelerateResult<()> {
        let bounds: serde_json::Value =
            serde_json::from_str(&self.raw_window_bounds().await?).unwrap_or_default();
        let width = bounds.get("width").and_then(|v| v.as_i64()).unwrap_or(800);
        let height = bounds.get("height").and_then(|v| v.as_i64()).unwrap_or(600);
        self.set_window_bounds(x, y, width, height).await
    }

    // -----------------------------------------------------------------------
    // Network interception (Fetch domain)
    // -----------------------------------------------------------------------

    async fn ensure_interception(&self) {
        let mut guard = self.interception_task.lock().await;
        if guard.is_none() {
            *guard = Some(tokio::spawn(run_interception(
                Arc::clone(&self.client),
                self.session_id.clone(),
                Arc::clone(&self.routes),
                Arc::clone(&self.requests),
                Arc::clone(&self.credentials),
            )));
        }
    }

    /// Enables or disables request interception for this page.
    pub async fn set_request_interception(&self, enabled: bool) -> XcelerateResult<()> {
        if enabled {
            self.ensure_interception().await;
        } else {
            if let Some(handle) = self.interception_task.lock().await.take() {
                handle.abort();
            }
            self.client
                .execute_raw_with_session(
                    Some(&self.session_id),
                    "Fetch.disable",
                    serde_json::json!({}),
                )
                .await?;
        }
        Ok(())
    }

    /// Adds a route rule. `action` is `continue`, `abort`, or `fulfill`.
    pub async fn route(
        &self,
        pattern: String,
        action: String,
        body: Option<String>,
        content_type: Option<String>,
    ) -> XcelerateResult<()> {
        self.routes.lock().await.push(RouteRule {
            pattern,
            action,
            body,
            content_type,
        });
        self.ensure_interception().await;
        Ok(())
    }

    /// Aborts every request matching `pattern`.
    pub async fn route_abort(&self, pattern: String) -> XcelerateResult<()> {
        self.route(pattern, "abort".to_string(), None, None).await
    }

    /// Fulfills every request matching `pattern` with `body`.
    pub async fn route_fulfill(
        &self,
        pattern: String,
        body: String,
        content_type: Option<String>,
    ) -> XcelerateResult<()> {
        self.route(pattern, "fulfill".to_string(), Some(body), content_type)
            .await
    }

    /// Removes the routes registered for `pattern`.
    pub async fn unroute(&self, pattern: String) -> XcelerateResult<()> {
        self.routes
            .lock()
            .await
            .retain(|rule| rule.pattern != pattern);
        Ok(())
    }

    /// Removes every route rule.
    pub async fn unroute_all(&self) -> XcelerateResult<()> {
        self.routes.lock().await.clear();
        Ok(())
    }

    /// Returns the intercepted requests seen so far as JSON.
    pub async fn requests(&self) -> XcelerateResult<String> {
        let list = self.requests.lock().await;
        Ok(serde_json::Value::Array(list.clone()).to_string())
    }

    /// Returns the most recent intercepted request as JSON.
    pub async fn request(&self) -> XcelerateResult<String> {
        let list = self.requests.lock().await;
        Ok(list
            .last()
            .cloned()
            .unwrap_or(serde_json::Value::Null)
            .to_string())
    }

    /// Clears the recorded intercepted requests.
    pub async fn clear_requests(&self) {
        self.requests.lock().await.clear();
    }

    /// Sets the credentials used to answer HTTP auth challenges.
    pub async fn authenticate(&self, username: String, password: String) -> XcelerateResult<()> {
        *self.credentials.lock().await = Some((username, password));
        self.ensure_interception().await;
        Ok(())
    }

    /// Enables or disables input drag interception.
    pub async fn set_drag_interception(&self, enabled: bool) -> XcelerateResult<()> {
        self.drag_interception
            .store(enabled, std::sync::atomic::Ordering::SeqCst);
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Input.setInterceptDrags",
                serde_json::json!({ "enabled": enabled }),
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Whether drag interception is enabled.
    pub async fn is_drag_interception_enabled(&self) -> bool {
        self.drag_interception
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    // -----------------------------------------------------------------------
    // Frames
    // -----------------------------------------------------------------------

    /// Returns every frame in the page as a JSON array.
    pub async fn frames(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Page.getFrameTree",
                serde_json::json!({}),
            )
            .await?;
        let mut frames = Vec::new();
        if let Some(tree) = res.get("frameTree") {
            collect_frames(tree, &mut frames);
        }
        Ok(serde_json::Value::Array(frames).to_string())
    }

    /// Returns the main frame as JSON.
    pub async fn main_frame(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Page.getFrameTree",
                serde_json::json!({}),
            )
            .await?;
        Ok(res
            .get("frameTree")
            .and_then(|tree| tree.get("frame"))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
            .to_string())
    }

    /// Returns the frame matching an id or name as JSON (or null).
    pub async fn frame(&self, frame_id: String) -> XcelerateResult<String> {
        let list: serde_json::Value =
            serde_json::from_str(&self.frames().await?).unwrap_or_default();
        let found = list
            .as_array()
            .and_then(|frames| {
                frames.iter().find(|frame| {
                    frame
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|id| id == frame_id)
                        .unwrap_or(false)
                        || frame
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(|name| name == frame_id)
                            .unwrap_or(false)
                })
            })
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        Ok(found.to_string())
    }

    /// The CDP target id backing this page.
    pub fn target_id(&self) -> String {
        self.target_id.clone()
    }

    /// Waits for the first XPath match to appear.
    pub async fn wait_for_xpath(
        self: Arc<Self>,
        xpath: String,
        timeout_ms: u64,
    ) -> XcelerateResult<Arc<Element>> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms.max(1));
        loop {
            if let Ok(element) = self.clone().query_selector_xpath(xpath.clone()).await {
                return Ok(element);
            }
            if start.elapsed() >= timeout {
                return Err(XcelerateError::NotFound(format!(
                    "Timeout waiting for xpath: {xpath}"
                )));
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    /// Reads a local file and injects it as an init script.
    pub async fn inject_file(&self, path: String) -> XcelerateResult<String> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| XcelerateError::NotFound(format!("failed to read {path}: {e}")))?;
        self.add_script_to_evaluate_on_new_document(content).await
    }

    /// Removes an init script by identifier.
    pub async fn remove_script(&self, identifier: String) -> XcelerateResult<()> {
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Page.removeScriptToEvaluateOnNewDocument",
                serde_json::json!({ "identifier": identifier }),
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Returns the main frame's name.
    pub async fn frame_name(&self) -> XcelerateResult<String> {
        let value: serde_json::Value =
            serde_json::from_str(&self.main_frame().await?).unwrap_or_default();
        Ok(value
            .get("name")
            .and_then(|name| name.as_str())
            .unwrap_or_default()
            .to_string())
    }

    /// Registers fulfill routes for every entry in a HAR file.
    pub async fn route_from_har(&self, path: String) -> XcelerateResult<()> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| XcelerateError::NotFound(format!("failed to read {path}: {e}")))?;
        let har: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let entries = har
            .get("log")
            .and_then(|log| log.get("entries"))
            .and_then(|entries| entries.as_array())
            .cloned()
            .unwrap_or_default();
        for entry in entries {
            let url = entry
                .get("request")
                .and_then(|request| request.get("url"))
                .and_then(|url| url.as_str())
                .unwrap_or_default()
                .to_string();
            let text = entry
                .get("response")
                .and_then(|response| response.get("content"))
                .and_then(|content| content.get("text"))
                .and_then(|text| text.as_str())
                .unwrap_or_default()
                .to_string();
            if !url.is_empty() {
                self.route_fulfill(url, text, None).await?;
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Keyboard / touch / target activation
    // -----------------------------------------------------------------------

    /// Presses `key` on the focused element.
    pub async fn keyboard_press(&self, key: String) -> XcelerateResult<()> {
        self.call_json(
            "function(k){const e=document.activeElement||document.body;e.dispatchEvent(new KeyboardEvent('keydown',{key:k,bubbles:true}));e.dispatchEvent(new KeyboardEvent('keyup',{key:k,bubbles:true}));}"
                .to_string(),
            serde_json::json!([key]).to_string(),
        )
        .await?;
        Ok(())
    }

    /// Dispatches a keydown event for `key`.
    pub async fn keyboard_down(&self, key: String) -> XcelerateResult<()> {
        self.call_json(
            "function(k){const e=document.activeElement||document.body;e.dispatchEvent(new KeyboardEvent('keydown',{key:k,bubbles:true}));}"
                .to_string(),
            serde_json::json!([key]).to_string(),
        )
        .await?;
        Ok(())
    }

    /// Dispatches a keyup event for `key`.
    pub async fn keyboard_up(&self, key: String) -> XcelerateResult<()> {
        self.call_json(
            "function(k){const e=document.activeElement||document.body;e.dispatchEvent(new KeyboardEvent('keyup',{key:k,bubbles:true}));}"
                .to_string(),
            serde_json::json!([key]).to_string(),
        )
        .await?;
        Ok(())
    }

    /// Types `text` into the focused element.
    pub async fn keyboard_type(&self, text: String) -> XcelerateResult<()> {
        self.call_json(
            "function(t){const e=document.activeElement||document.body;e.dispatchEvent(new KeyboardEvent('keydown',{key:t,bubbles:true}));if(e.value!==undefined){e.value+=t;e.dispatchEvent(new Event('input',{bubbles:true}));}e.dispatchEvent(new KeyboardEvent('keyup',{key:t,bubbles:true}));}"
                .to_string(),
            serde_json::json!([text]).to_string(),
        )
        .await?;
        Ok(())
    }

    /// Dispatches a touch tap at (x, y).
    pub async fn touch_tap(&self, x: f64, y: f64) -> XcelerateResult<()> {
        let points = serde_json::json!([{ "x": x, "y": y }]);
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Input.dispatchTouchEvent",
                serde_json::json!({ "type": "touchStart", "touchPoints": points }),
            )
            .await?;
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Input.dispatchTouchEvent",
                serde_json::json!({ "type": "touchEnd", "touchPoints": [] }),
            )
            .await?;
        Ok(())
    }

    /// Activates the given target (window/tab).
    pub async fn activate_target(&self, target_id: String) -> XcelerateResult<()> {
        self.client
            .execute_raw(
                "Target.activateTarget",
                serde_json::json!({ "targetId": target_id }),
            )
            .await?;
        Ok(())
    }

    /// Activates this page's target.
    pub async fn activate(&self) -> XcelerateResult<()> {
        self.activate_target(self.target_id.clone()).await
    }

    /// Accepts or dismisses the active JavaScript dialog.
    pub async fn handle_js_dialog(
        &self,
        accept: bool,
        prompt_text: Option<String>,
    ) -> XcelerateResult<()> {
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Page.handleJavaScriptDialog",
                serde_json::json!({ "accept": accept, "promptText": prompt_text }),
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Starts CDP tracing on this page's session.
    pub async fn start_tracing(&self) -> XcelerateResult<()> {
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Tracing.start",
                serde_json::json!({ "categories": "*", "transferMode": "ReportEvents" }),
            )
            .await?;
        Ok(())
    }

    /// Stops CDP tracing on this page's session.
    pub async fn stop_tracing(&self) -> XcelerateResult<()> {
        self.client
            .execute_raw_with_session(Some(&self.session_id), "Tracing.end", serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Returns the page PDF as a base64 string.
    pub async fn create_pdf_stream(&self) -> XcelerateResult<String> {
        use base64::{Engine as _, engine::general_purpose};
        let data = self.pdf().await?;
        Ok(general_purpose::STANDARD.encode(data))
    }

    /// Overrides media features (JSON array of `{name,value}`).
    pub async fn set_emulated_media_features(&self, features_json: String) -> XcelerateResult<()> {
        let features: serde_json::Value =
            serde_json::from_str(&features_json).unwrap_or(serde_json::json!([]));
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Emulation.setEmulatedMedia",
                serde_json::json!({ "features": features }),
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Overrides the idle state.
    pub async fn emulate_idle_state(
        &self,
        is_user_active: bool,
        is_screen_unlocked: bool,
    ) -> XcelerateResult<()> {
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Emulation.setIdleOverride",
                serde_json::json!({
                    "isUserActive": is_user_active,
                    "isScreenUnlocked": is_screen_unlocked
                }),
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Starts a PNG screencast.
    pub async fn start_screencast(&self) -> XcelerateResult<()> {
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Page.startScreencast",
                serde_json::json!({ "format": "png", "everyNthFrame": 1 }),
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Stops the screencast.
    pub async fn stop_screencast(&self) -> XcelerateResult<()> {
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Page.stopScreencast",
                serde_json::json!({}),
            )
            .await
            .map(|_| ())
            .map_err(XcelerateError::from)
    }

    /// Starts JS coverage collection.
    pub async fn coverage_start_js(&self) -> XcelerateResult<()> {
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Profiler.enable",
                serde_json::json!({}),
            )
            .await?;
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Profiler.startPreciseCoverage",
                serde_json::json!({ "detailed": true }),
            )
            .await?;
        Ok(())
    }

    /// Stops JS coverage collection and returns the result as JSON.
    pub async fn coverage_stop_js(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Profiler.takePreciseCoverage",
                serde_json::json!({}),
            )
            .await?;
        let _ = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Profiler.stopPreciseCoverage",
                serde_json::json!({}),
            )
            .await;
        Ok(res.to_string())
    }

    /// Starts CSS coverage collection.
    pub async fn coverage_start_css(&self) -> XcelerateResult<()> {
        self.client
            .execute_raw_with_session(Some(&self.session_id), "CSS.enable", serde_json::json!({}))
            .await?;
        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "CSS.startRuleUsageTracking",
                serde_json::json!({}),
            )
            .await?;
        Ok(())
    }

    /// Stops CSS coverage collection and returns the result as JSON.
    pub async fn coverage_stop_css(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "CSS.stopRuleUsageTracking",
                serde_json::json!({}),
            )
            .await?;
        Ok(res.to_string())
    }

    /// Returns cookies + localStorage as a storage-state JSON object.
    pub async fn storage_state(&self) -> XcelerateResult<String> {
        let cookies: serde_json::Value =
            serde_json::from_str(&self.cookies().await?).unwrap_or(serde_json::json!([]));
        let local: serde_json::Value = serde_json::from_str(
            &self
                .evaluate_json("Object.fromEntries(Object.entries(localStorage))".to_string())
                .await?,
        )
        .unwrap_or(serde_json::json!({}));
        Ok(serde_json::json!({
            "cookies": cookies,
            "origins": [{ "origin": self.url().await.unwrap_or_default(), "localStorage": local }]
        })
        .to_string())
    }

    /// Restores cookies + localStorage from a storage-state JSON object.
    pub async fn set_storage_state(&self, state_json: String) -> XcelerateResult<()> {
        let state: serde_json::Value =
            serde_json::from_str(&state_json).unwrap_or(serde_json::json!({}));
        if let Some(cookies) = state.get("cookies").and_then(|value| value.as_array()) {
            for cookie in cookies {
                let _ = self
                    .client
                    .execute_raw_with_session(
                        Some(&self.session_id),
                        "Network.setCookie",
                        cookie.clone(),
                    )
                    .await;
            }
        }
        if let Some(origins) = state.get("origins").and_then(|value| value.as_array()) {
            for origin in origins {
                if let Some(storage) = origin.get("localStorage") {
                    let script = format!(
                        "Object.entries({storage}).forEach(function(e){{localStorage.setItem(e[0], e[1]);}});"
                    );
                    self.evaluate_json(script).await?;
                }
            }
        }
        Ok(())
    }

    /// The default timeout (ms) used by the waiting helpers. A stored value of
    /// `0` means "no timeout" and is mapped to the largest representable wait.
    fn default_timeout(&self) -> u64 {
        match self
            .default_timeout_ms
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            0 => u64::MAX,
            millis => millis,
        }
    }

    /// Sets the default timeout (ms) applied by [`Page::wait_for_selector`],
    /// [`Page::wait_for_navigation`], and [`Page::wait_for_event_default`].
    /// As in Playwright, `0` disables the timeout.
    pub async fn set_default_timeout(&self, milliseconds: f64) -> XcelerateResult<()> {
        let millis = if milliseconds.is_finite() && milliseconds > 0.0 {
            milliseconds as u64
        } else {
            0
        };
        self.default_timeout_ms
            .store(millis, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// Returns the stored default timeout (ms).
    pub async fn get_default_timeout(&self) -> XcelerateResult<f64> {
        Ok(self
            .default_timeout_ms
            .load(std::sync::atomic::Ordering::SeqCst) as f64)
    }
}

/// Recursively flattens a CDP frame tree into a list of frame objects.
fn collect_frames(node: &serde_json::Value, out: &mut Vec<serde_json::Value>) {
    if let Some(frame) = node.get("frame") {
        out.push(frame.clone());
    }
    if let Some(children) = node.get("childFrames").and_then(|value| value.as_array()) {
        for child in children {
            collect_frames(child, out);
        }
    }
}

/// Reads the indexed elements of a JS array remote object as [`Element`] handles.
///
/// Shared by `Page::query_selector_all` and `Element::query_selector_all` so both
/// resolve an entire node list with a single `Runtime.getProperties` call.
pub(crate) async fn collect_elements(
    client: &CdpClient,
    session_id: &str,
    page: &Arc<Page>,
    object_id: js_protocol::runtime::RemoteObjectId<'_>,
) -> XcelerateResult<Vec<Arc<Element>>> {
    let properties = client
        .execute_with_session(
            Some(session_id),
            js_protocol::runtime::GetPropertiesParams {
                object_id,
                own_properties: Some(true),
                ..Default::default()
            },
        )
        .await?;

    // Array indices can be returned in any order; sort so the result matches DOM order.
    let mut indexed: Vec<(usize, Arc<Element>)> = Vec::new();
    for descriptor in properties.result {
        if let Ok(index) = descriptor.name.parse::<usize>()
            && let Some(object_id) = descriptor.value.and_then(|value| value.object_id)
        {
            indexed.push((
                index,
                Arc::new(Element {
                    page: Arc::clone(page),
                    object_id: object_id.into_owned(),
                }),
            ));
        }
    }
    indexed.sort_by_key(|(index, _)| *index);
    Ok(indexed.into_iter().map(|(_, element)| element).collect())
}
