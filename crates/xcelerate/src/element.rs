use crate::error::{XcelerateError, XcelerateResult};
use crate::page::{Lcg, Page};
use browser_protocol::dom::SetFileInputFilesParams;
use browser_protocol::input::DispatchKeyEventParams;
use browser_protocol::page::{CaptureScreenshotParams, Viewport};
use std::sync::Arc;

/// Represents an HTML element in the DOM.
#[derive(uniffi::Object)]
pub struct Element {
    pub(crate) page: Arc<Page>,
    pub(crate) object_id: String,
}

#[uniffi::export(async_runtime = "tokio")]
impl Element {
    /// Clicks the element.
    pub async fn click(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
        self.call_js("function() { this.click(); }".to_string())
            .await?;
        Ok(self)
    }

    pub async fn type_text(self: Arc<Self>, text: String) -> XcelerateResult<Arc<Self>> {
        // 1. Focus the element first
        self.clone().focus().await?;

        // 2. Dispatch key events for each character
        for c in text.chars() {
            let mut params = browser_protocol::input::DispatchKeyEventParams {
                type_: "char".into(),
                ..Default::default()
            };
            params.text = Some(c.to_string().into());
            params.unmodified_text = Some(c.to_string().into());

            self.page
                .client
                .execute_with_session(Some(&self.page.session_id), params)
                .await?;

            // Subtle delay to mimic human typing
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        Ok(self)
    }

    /// Hovers over the element.
    pub async fn hover(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
        self.call_js(
            "function() { this.dispatchEvent(new MouseEvent('mouseover', { bubbles: true })); }"
                .to_string(),
        )
        .await?;
        Ok(self)
    }

    /// Clicks the element using realistic mouse movement and CDP input events.
    pub async fn click_stealth(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
        let js = "function() {
            this.scrollIntoView({ block: 'center', inline: 'center' });
            const rect = this.getBoundingClientRect();
            return JSON.stringify({
                x: rect.left,
                y: rect.top,
                width: rect.width,
                height: rect.height
            });
        }"
        .to_string();

        let res = self.call_js(js).await?;
        let val_str = res
            .result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or(crate::error::XcelerateError::InternalError)?;

        #[derive(serde::Deserialize)]
        struct Rect {
            x: f64,
            y: f64,
            width: f64,
            height: f64,
        }

        let rect: Rect = serde_json::from_str(&val_str)
            .map_err(|e| crate::error::XcelerateError::SerdeError(e.to_string()))?;

        let mut rng = Lcg::new();
        let target_x = rect.x + rect.width * 0.15 + rng.range(0.0, rect.width * 0.7);
        let target_y = rect.y + rect.height * 0.15 + rng.range(0.0, rect.height * 0.7);

        self.page.clone().click_mouse(target_x, target_y).await?;

        Ok(self)
    }

    /// Hovers over the element using realistic mouse movement.
    pub async fn hover_stealth(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
        let js = "function() {
            this.scrollIntoView({ block: 'center', inline: 'center' });
            const rect = this.getBoundingClientRect();
            return JSON.stringify({
                x: rect.left,
                y: rect.top,
                width: rect.width,
                height: rect.height
            });
        }"
        .to_string();

        let res = self.call_js(js).await?;
        let val_str = res
            .result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or(crate::error::XcelerateError::InternalError)?;

        #[derive(serde::Deserialize)]
        struct Rect {
            x: f64,
            y: f64,
            width: f64,
            height: f64,
        }

        let rect: Rect = serde_json::from_str(&val_str)
            .map_err(|e| crate::error::XcelerateError::SerdeError(e.to_string()))?;

        let mut rng = Lcg::new();
        let target_x = rect.x + rect.width * 0.15 + rng.range(0.0, rect.width * 0.7);
        let target_y = rect.y + rect.height * 0.15 + rng.range(0.0, rect.height * 0.7);

        self.page.clone().move_mouse(target_x, target_y).await?;

        Ok(self)
    }

    /// Focuses the element.
    pub async fn focus(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
        self.call_js("function() { this.focus(); }".to_string())
            .await?;
        Ok(self)
    }

    /// Returns the visible text of the element.
    pub async fn text(&self) -> XcelerateResult<String> {
        let res = self
            .call_js("function() { return this.innerText; }".to_string())
            .await?;
        Ok(res
            .result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default())
    }

    /// Returns the value of a specific attribute.
    pub async fn attribute(&self, name: String) -> XcelerateResult<Option<String>> {
        let js = format!("function() {{ return this.getAttribute('{}'); }}", name);
        let res = self.call_js(js).await?;
        Ok(res
            .result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    /// Returns the inner HTML of the element.
    pub async fn inner_html(&self) -> XcelerateResult<String> {
        let res = self
            .call_js("function() { return this.innerHTML; }".to_string())
            .await?;
        Ok(res
            .result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default())
    }

    // -----------------------------------------------------------------------
    // Generic JS / CDP bridge (see `Page` for the rationale).
    // -----------------------------------------------------------------------

    /// Calls a function on this element; the result is returned as JSON text.
    pub async fn evaluate_json(&self, function: String) -> XcelerateResult<String> {
        let res = self.call_js(function).await?;
        Ok(res
            .result
            .value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string()))
    }

    /// Calls a function on this element and coerces the result to a string.
    pub async fn evaluate_string(&self, function: String) -> XcelerateResult<String> {
        let res = self.call_js(function).await?;
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

    /// Calls a function on this element and coerces the result to a bool.
    pub async fn evaluate_bool(&self, function: String) -> XcelerateResult<bool> {
        let res = self.call_js(function).await?;
        Ok(res.result.value.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// Focuses the element and presses a key.
    pub async fn press(self: Arc<Self>, key: String) -> XcelerateResult<()> {
        self.clone().focus().await?;
        let (virtual_key, code, name) = key_info(&key);
        let text = if key.chars().count() == 1 {
            Some(key.clone())
        } else {
            None
        };

        let mut down = DispatchKeyEventParams {
            type_: "keyDown".into(),
            ..Default::default()
        };
        down.key = Some(name.clone().into());
        down.code = Some(code.clone().into());
        down.windows_virtual_key_code = Some(virtual_key);
        down.native_virtual_key_code = Some(virtual_key);
        if let Some(text) = &text {
            down.text = Some(text.clone().into());
            down.unmodified_text = Some(text.clone().into());
        }
        self.page
            .client
            .execute_with_session(Some(&self.page.session_id), down)
            .await?;

        let mut up = DispatchKeyEventParams {
            type_: "keyUp".into(),
            ..Default::default()
        };
        up.key = Some(name.into());
        up.code = Some(code.into());
        up.windows_virtual_key_code = Some(virtual_key);
        up.native_virtual_key_code = Some(virtual_key);
        self.page
            .client
            .execute_with_session(Some(&self.page.session_id), up)
            .await?;
        Ok(())
    }

    /// Selects options by value or label on this `<select>` element.
    pub async fn select_option(&self, values_json: String) -> XcelerateResult<()> {
        let values: serde_json::Value = serde_json::from_str(&values_json)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let js = format!(
            "function() {{ const wanted = {values}.map(String); for (const option of this.options) {{\n                option.selected = wanted.includes(option.value) || wanted.includes(option.text);\n            }} this.dispatchEvent(new Event('change', {{ bubbles: true }})); return true; }}"
        );
        self.evaluate_json(js).await?;
        Ok(())
    }

    /// Sets the files of this `<input type="file">` element.
    pub async fn set_input_files(&self, files_json: String) -> XcelerateResult<()> {
        let files: Vec<String> = serde_json::from_str(&files_json)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let params = SetFileInputFilesParams {
            files: files.into_iter().map(Into::into).collect(),
            node_id: None,
            backend_node_id: None,
            object_id: Some(self.object_id.clone().into()),
        };
        self.page
            .client
            .execute_with_session(Some(&self.page.session_id), params)
            .await?;
        Ok(())
    }

    /// Calls a JS function on this element with JSON-encoded arguments.
    pub async fn call_json(&self, function: String, args_json: String) -> XcelerateResult<String> {
        let values: Vec<serde_json::Value> = serde_json::from_str(&args_json).unwrap_or_default();
        let arguments = values
            .into_iter()
            .map(|value| js_protocol::runtime::CallArgument {
                value: Some(value),
                unserializable_value: None,
                object_id: None,
            })
            .collect();
        let res = self
            .page
            .client
            .execute_with_session(
                Some(&self.page.session_id),
                js_protocol::runtime::CallFunctionOnParams {
                    function_declaration: function.into(),
                    object_id: Some(self.object_id.clone().into()),
                    arguments: Some(arguments),
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

    /// Like [`Element::call_json`] but coerces the result to a string.
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

    /// Like [`Element::call_json`] but coerces the result to a bool.
    pub async fn call_bool(&self, function: String, args_json: String) -> XcelerateResult<bool> {
        let json = self.call_json(function, args_json).await?;
        Ok(serde_json::from_str::<serde_json::Value>(&json)
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    /// Returns the first descendant matching `selector` as an [`Element`].
    pub async fn query_selector(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Arc<Element>> {
        let quoted = serde_json::to_string(&selector)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let res = self
            .page
            .client
            .execute_with_session(
                Some(&self.page.session_id),
                js_protocol::runtime::CallFunctionOnParams {
                    function_declaration: format!(
                        "function(){{return this.querySelector({quoted});}}"
                    )
                    .into(),
                    object_id: Some(self.object_id.clone().into()),
                    return_by_value: Some(false),
                    ..Default::default()
                },
            )
            .await?;
        if let Some(object_id) = res.result.object_id {
            Ok(Arc::new(Element {
                page: self.page.clone(),
                object_id: object_id.into_owned(),
            }))
        } else {
            Err(XcelerateError::NotFound(selector))
        }
    }

    /// Calls a JS function on this element and returns the resulting node.
    pub async fn evaluate_handle(
        self: Arc<Self>,
        function: String,
    ) -> XcelerateResult<Arc<Element>> {
        let res = self
            .page
            .client
            .execute_with_session(
                Some(&self.page.session_id),
                js_protocol::runtime::CallFunctionOnParams {
                    function_declaration: function.into(),
                    object_id: Some(self.object_id.clone().into()),
                    return_by_value: Some(false),
                    ..Default::default()
                },
            )
            .await?;
        if let Some(object_id) = res.result.object_id {
            Ok(Arc::new(Element {
                page: self.page.clone(),
                object_id: object_id.into_owned(),
            }))
        } else {
            Err(XcelerateError::NotFound(
                "evaluate_handle returned null".to_string(),
            ))
        }
    }

    /// Returns every descendant matching `selector`.
    ///
    /// Resolves the whole node list with a single `Runtime.getProperties` call
    /// rather than one `evaluate` per match.
    pub async fn query_selector_all(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Vec<Arc<Element>>> {
        let quoted = serde_json::to_string(&selector)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let res = self
            .page
            .client
            .execute_with_session(
                Some(&self.page.session_id),
                js_protocol::runtime::CallFunctionOnParams {
                    function_declaration: format!(
                        "function(){{return Array.from(this.querySelectorAll({quoted}));}}"
                    )
                    .into(),
                    object_id: Some(self.object_id.clone().into()),
                    return_by_value: Some(false),
                    ..Default::default()
                },
            )
            .await?;
        match res.result.object_id {
            Some(object_id) => {
                crate::page::collect_elements(
                    &self.page.client,
                    &self.page.session_id,
                    &self.page,
                    object_id,
                )
                .await
            }
            None => Ok(Vec::new()),
        }
    }

    /// Captures a PNG screenshot cropped to this element.
    pub async fn screenshot(&self) -> XcelerateResult<Vec<u8>> {
        let rect: serde_json::Value = serde_json::from_str(
            &self
                .evaluate_json(
                    "function(){const r=this.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}"
                        .to_string(),
                )
                .await?,
        )
        .unwrap_or_default();
        let clip = Viewport {
            x: rect.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
            y: rect.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
            width: rect.get("width").and_then(|v| v.as_f64()).unwrap_or(1.0),
            height: rect.get("height").and_then(|v| v.as_f64()).unwrap_or(1.0),
            scale: 1.0,
        };
        let res = self
            .page
            .client
            .execute_with_session(
                Some(&self.page.session_id),
                CaptureScreenshotParams {
                    clip: Some(clip),
                    ..Default::default()
                },
            )
            .await?;
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD
            .decode(res.data.as_bytes())
            .map_err(|e| XcelerateError::SerdeError(format!("Base64 decode failed: {e}")))
    }

    /// Captures a base64 PNG screenshot cropped to this element.
    pub async fn screenshot_base64(&self) -> XcelerateResult<String> {
        use base64::{Engine as _, engine::general_purpose};
        let data = self.screenshot().await?;
        Ok(general_purpose::STANDARD.encode(data))
    }

    /// Number of elements this handle represents (always 1).
    pub async fn count(&self) -> XcelerateResult<i64> {
        Ok(1)
    }

    /// Runs a JS function against the first descendant matching `selector`.
    pub async fn call_on_selector(
        &self,
        selector: String,
        expression: String,
    ) -> XcelerateResult<String> {
        self.call_json(
            "function(sel,src){const el=this.querySelector(sel);if(!el)return null;return (new Function('el','return ('+src+')(el);'))(el);}"
                .to_string(),
            serde_json::json!([selector, expression]).to_string(),
        )
        .await
    }

    /// Runs a JS function against every descendant matching `selector`.
    pub async fn call_on_selector_all(
        &self,
        selector: String,
        expression: String,
    ) -> XcelerateResult<String> {
        self.call_json(
            "function(sel,src){const els=Array.from(this.querySelectorAll(sel));const fn=new Function('el','return ('+src+')(el);');return els.map(fn);}"
                .to_string(),
            serde_json::json!([selector, expression]).to_string(),
        )
        .await
    }

    /// Finds a descendant by attribute value.
    pub async fn query_selector_attr(
        self: Arc<Self>,
        attribute: String,
        value: String,
    ) -> XcelerateResult<Arc<Element>> {
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        self.query_selector(format!("[{attribute}=\"{escaped}\"]"))
            .await
    }

    /// Finds a descendant whose text contains `text`.
    pub async fn get_by_text(self: Arc<Self>, text: String) -> XcelerateResult<Arc<Element>> {
        let quoted =
            serde_json::to_string(&text).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_handle(format!(
            "function(){{return Array.from(this.querySelectorAll('*')).find(function(e){{return e.textContent.includes({quoted});}});}}"
        ))
        .await
    }

    /// Finds a descendant by ARIA role.
    pub async fn get_by_role(self: Arc<Self>, role: String) -> XcelerateResult<Arc<Element>> {
        self.query_selector_attr("role".to_string(), role).await
    }

    /// Finds a descendant form control by its `<label>` text.
    pub async fn get_by_label(self: Arc<Self>, label: String) -> XcelerateResult<Arc<Element>> {
        let quoted =
            serde_json::to_string(&label).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_handle(format!(
            "function(){{const labels=Array.from(this.querySelectorAll('label')).filter(function(x){{return x.textContent.trim().includes({quoted});}});if(!labels.length)return null;const lab=labels[0];if(lab.control)return lab.control;const f=lab.getAttribute('for');return f?document.getElementById(f):null;}}"
        ))
        .await
    }

    /// Finds a descendant matching an XPath expression.
    pub async fn query_selector_xpath(
        self: Arc<Self>,
        xpath: String,
    ) -> XcelerateResult<Arc<Element>> {
        let quoted =
            serde_json::to_string(&xpath).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        self.evaluate_handle(format!(
            "function(){{return document.evaluate({quoted}, this, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;}}"
        ))
        .await
    }

    /// Returns this element's enumerable properties as a JSON object.
    pub async fn get_properties(&self) -> XcelerateResult<String> {
        self.call_json(
            "function(){const o={};for(const k in this){try{const v=this[k];o[k]=(v===null||v===undefined)?null:String(v);}catch(e){}}return o;}"
                .to_string(),
            "[]".to_string(),
        )
        .await
    }

    /// Releases the underlying remote object handle.
    pub async fn dispose(&self) -> XcelerateResult<()> {
        self.page
            .client
            .execute_raw_with_session(
                Some(&self.page.session_id),
                "Runtime.releaseObject",
                serde_json::json!({ "objectId": self.object_id }),
            )
            .await?;
        Ok(())
    }

    /// Waits for a descendant matching `selector` to appear.
    pub async fn wait_for_selector(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Arc<Element>> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        loop {
            if let Ok(element) = self.clone().query_selector(selector.clone()).await {
                return Ok(element);
            }
            if start.elapsed() >= timeout {
                return Err(XcelerateError::NotFound(format!(
                    "Timeout waiting for selector: {selector}"
                )));
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    }
}

impl Element {
    /// Helper to call JS on this element.
    async fn call_js(
        &self,
        js: String,
    ) -> XcelerateResult<js_protocol::runtime::CallFunctionOnReturns<'static>> {
        self.page
            .client
            .execute_with_session(
                Some(&self.page.session_id),
                js_protocol::runtime::CallFunctionOnParams {
                    function_declaration: js.into(),
                    object_id: Some(self.object_id.clone().into()),
                    ..Default::default()
                },
            )
            .await
            .map_err(XcelerateError::from)
    }
}

/// Maps a key name to `(windowsVirtualKeyCode, code, key)` for CDP key events.
fn key_info(key: &str) -> (i64, String, String) {
    match key {
        "Enter" | "NumpadEnter" => (13, "Enter".to_string(), "Enter".to_string()),
        "Tab" => (9, "Tab".to_string(), "Tab".to_string()),
        "Escape" => (27, "Escape".to_string(), "Escape".to_string()),
        "Backspace" => (8, "Backspace".to_string(), "Backspace".to_string()),
        "Delete" => (46, "Delete".to_string(), "Delete".to_string()),
        "ArrowUp" => (38, "ArrowUp".to_string(), "ArrowUp".to_string()),
        "ArrowDown" => (40, "ArrowDown".to_string(), "ArrowDown".to_string()),
        "ArrowLeft" => (37, "ArrowLeft".to_string(), "ArrowLeft".to_string()),
        "ArrowRight" => (39, "ArrowRight".to_string(), "ArrowRight".to_string()),
        "Home" => (36, "Home".to_string(), "Home".to_string()),
        "End" => (35, "End".to_string(), "End".to_string()),
        "PageUp" => (33, "PageUp".to_string(), "PageUp".to_string()),
        "PageDown" => (34, "PageDown".to_string(), "PageDown".to_string()),
        " " | "Space" => (32, "Space".to_string(), " ".to_string()),
        other => {
            let upper = other.to_uppercase();
            let code = if upper.chars().count() == 1 {
                format!("Key{upper}")
            } else {
                other.to_string()
            };
            (0, code, other.to_string())
        }
    }
}
