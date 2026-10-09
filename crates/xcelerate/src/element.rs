use crate::error::{XcelerateError, XcelerateResult};
use crate::page::{Lcg, Page};
use browser_protocol::dom::SetFileInputFilesParams;
use browser_protocol::input::DispatchKeyEventParams;
use browser_protocol::page::{CaptureScreenshotParams, Viewport};
use std::sync::Arc;

/// Shadow-piercing descendant search used by [`Element::query_selector`].
///
/// `querySelector` cannot see into a shadow root, so web components are
/// invisible to it; this walks open shadow roots as well. Cross-origin iframes
/// stay out of scope (page script cannot reach them) - see
/// [`Element::query_selector_all_frames`] for a same-origin variant.
pub(crate) const JS_QUERY_ONE: &str = r#"function(sel){
  const visit=(scope,out)=>{ if(!scope||!scope.querySelectorAll) return out;
    for(const el of scope.querySelectorAll(sel)) out.push(el);
    for(const el of scope.querySelectorAll('*')){ if(el.shadowRoot) visit(el.shadowRoot,out); }
    return out; };
  const out=[]; visit(this,out); if(this.shadowRoot) visit(this.shadowRoot,out);
  return out[0]||null;
}"#;

/// [`JS_QUERY_ONE`] for every match (`Element::query_selector_all`).
pub(crate) const JS_QUERY_ALL: &str = r#"function(sel){
  const visit=(scope,out)=>{ if(!scope||!scope.querySelectorAll) return out;
    for(const el of scope.querySelectorAll(sel)) out.push(el);
    for(const el of scope.querySelectorAll('*')){ if(el.shadowRoot) visit(el.shadowRoot,out); }
    return out; };
  const out=[]; visit(this,out); if(this.shadowRoot) visit(this.shadowRoot,out);
  return out;
}"#;

/// Like [`JS_QUERY_ALL`] but also descends into same-origin iframe documents.
const JS_QUERY_ALL_FRAMES: &str = r#"function(sel){
  const visit=(scope,out)=>{ if(!scope||!scope.querySelectorAll) return out;
    for(const el of scope.querySelectorAll(sel)) out.push(el);
    for(const el of scope.querySelectorAll('*')){ if(el.shadowRoot) visit(el.shadowRoot,out); if(el.contentDocument) visit(el.contentDocument,out); }
    return out; };
  const out=[]; visit(this,out); if(this.shadowRoot) visit(this.shadowRoot,out);
  return out;
}"#;

/// Shadow-piercing descendant whose text contains the argument (`Element::get_by_text`).
const JS_BY_TEXT: &str = r#"function(text){
  const visit=(scope,out)=>{ if(!scope||!scope.querySelectorAll) return out;
    for(const el of scope.querySelectorAll('*')){ out.push(el); if(el.shadowRoot) visit(el.shadowRoot,out); }
    return out; };
  const out=[]; visit(this,out); if(this.shadowRoot) visit(this.shadowRoot,out);
  for(const el of out){ if(el.textContent && el.textContent.includes(text)) return el; }
  return null;
}"#;

/// Resolves to the first shadow-piercing match, or null after the timeout. One CDP
/// call total: a document `MutationObserver` for the light DOM plus a slow rescan
/// for shadow roots (whose mutations never reach a document observer).
const JS_WAIT_FOR_SELECTOR: &str = r#"function(sel,ms){
  const visit=(scope,out)=>{ if(!scope||!scope.querySelectorAll) return out;
    for(const el of scope.querySelectorAll(sel)) out.push(el);
    for(const el of scope.querySelectorAll('*')){ if(el.shadowRoot) visit(el.shadowRoot,out); }
    return out; };
  const find=()=>{ const out=[]; visit(this,out); if(this.shadowRoot) visit(this.shadowRoot,out); return out[0]||null; };
  return new Promise((resolve)=>{
    const now=find(); if(now){ resolve(now); return; }
    let done=false, observer=null, rescan=null, timer=null;
    const finish=(el)=>{ if(done) return; done=true; if(observer) observer.disconnect(); clearInterval(rescan); clearTimeout(timer); resolve(el); };
    const scan=()=>{ const el=find(); if(el) finish(el); };
    observer=new MutationObserver(scan);
    observer.observe(document,{childList:true,subtree:true,attributes:true,characterData:true});
    rescan=setInterval(scan,100);
    timer=setTimeout(()=>finish(null),ms);
  });
}"#;

/// A self-contained XPath subset evaluator that walks the composed tree (open
/// shadow roots and same-origin iframes). Kept in its own file so the jsdom suite
/// in `.research/js` can test the exact bytes that ship; `include_str!` embeds it
/// verbatim. Its contract is documented at the top of the file.
const JS_XPATH: &str = include_str!("page/xpath_engine.js");

/// Native XPath fallback for expressions outside the subset: full language
/// support, but no shadow piercing. Run through [`Element::call_with_args`] so a
/// thrown `SyntaxError` surfaces as an error rather than a bogus result.
const JS_XPATH_NATIVE: &str = r#"function(xp){
  const r = document.evaluate(xp, this, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
  return r.singleNodeValue;
}"#;

/// [`JS_XPATH`]'s `all = true` counterpart for expressions outside the subset:
/// native XPath (full language, no shadow piercing) returning an Array of nodes,
/// which [`crate::page::collect_elements`] reads like any other node list.
const JS_XPATH_ALL_NATIVE: &str = r#"function(xp){ const r = document.evaluate(xp, this, null, XPathResult.ORDERED_NODE_SNAPSHOT_TYPE, null); const out = []; for (let i = 0; i < r.snapshotLength; i++) out.push(r.snapshotItem(i)); return out; }"#;

/// Computes an action point for [`Element::click_mouse`]/[`Element::hover_mouse`]
/// and drags, in top-level viewport coordinates.
///
/// `random` selects a jittered point inside the element (clicks) or the centre
/// (drags). The point is translated out of any same-origin frames so a real
/// pointer event lands on the element, and the element is hit-tested at that
/// point (descending shadow roots and same-origin frames) to report whether it is
/// actually on top. Failures of the hit test never report occlusion.
const JS_ACTION_POINT: &str = r#"function(random){
this.scrollIntoView({block:'center',inline:'center'});
var rect=this.getBoundingClientRect();
var style=getComputedStyle(this);
var visible=rect.width>0&&rect.height>0&&style.display!=='none'&&style.visibility!=='hidden'&&style.opacity!=='0';
var fx=random?(0.15+Math.random()*0.7):0.5;
var fy=random?(0.15+Math.random()*0.7):0.5;
var x=rect.left+rect.width*fx;var y=rect.top+rect.height*fy;
var win=window;var guard=0;
try{while(win!==win.top&&guard++<20){var fe=win.frameElement;if(!fe)break;var fr=fe.getBoundingClientRect();x+=fr.left+fe.clientLeft;y+=fr.top+fe.clientTop;win=fe.ownerDocument.defaultView;}}catch(e){}
var topX=x,topY=y;
var composedParent=function(n){if(!n)return null;if(n.parentNode)return n.parentNode;var r=n.getRootNode?n.getRootNode():null;return (r&&r.host)?r.host:null;};
var within=function(a,b){var n=b,g=0;while(n&&g++<100){if(n===a)return true;n=composedParent(n);}return false;};
var onTop=true;
try{if(win===window&&this.getRootNode()===document){var doc=win.document;var cur=doc.elementFromPoint(x,y);var cx=x,cy=y,g2=0;
while(cur&&g2++<50){if(cur.shadowRoot){var inner=cur.shadowRoot.elementFromPoint(cx,cy);if(inner&&inner!==cur){cur=inner;continue;}}
if(cur.tagName==='IFRAME'){var d=null;try{d=cur.contentDocument;}catch(e2){d=null;}if(d){var r2=cur.getBoundingClientRect();var ix=cx-(r2.left+cur.clientLeft);var iy=cy-(r2.top+cur.clientTop);var inner2=d.elementFromPoint(ix,iy);if(inner2){cur=inner2;cx=ix;cy=iy;continue;}}}
break;}
if(cur)onTop=within(this,cur)||within(cur,this);}}catch(e3){onTop=true;}
return {x:topX,y:topY,width:rect.width,height:rect.height,visible:visible,onTop:onTop};
}"#;

/// Deserialized result of [`JS_ACTION_POINT`].
#[derive(serde::Deserialize)]
struct ActionPoint {
    x: f64,
    y: f64,
    #[allow(dead_code)]
    width: f64,
    #[allow(dead_code)]
    height: f64,
    visible: bool,
    #[serde(rename = "onTop")]
    on_top: bool,
}

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

        // 2. Dispatch key events for each character. With human input on, the
        //    cadence varies per character and lingers after a space or
        //    punctuation, the way a person types; with it off, a short uniform
        //    delay keeps key events flowing without dragging the run out.
        let human = self.page.human();
        let mut rng = Lcg::new();
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

            let delay_ms = if human {
                let beat = rng.range(38.0, 96.0);
                // A short pause at word boundaries reads as human.
                if c.is_whitespace() {
                    (beat + rng.range(30.0, 90.0)) as u64
                } else {
                    beat as u64
                }
            } else {
                8
            };
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
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
    ///
    /// Fails with [`XcelerateError::NotFound`] if the element is not actionable
    /// (zero-size, `display:none`, `visibility:hidden` or fully transparent),
    /// rather than dispatching a click at coordinates that nothing occupies.
    pub async fn click_mouse(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
        // The shared probe gives a jittered point inside the element, translated
        // out of any same-origin frames, and hit-tests it so an occluding overlay
        // is reported instead of a click that silently does nothing.
        let (target_x, target_y, visible, on_top) = self.probe_action_point(true).await?;

        if !visible {
            return Err(crate::error::XcelerateError::NotFound(
                "element is not actionable (zero size, display:none, visibility:hidden or opacity:0)"
                    .to_string(),
            ));
        }
        if !on_top {
            return Err(crate::error::XcelerateError::NotFound(
                "element is covered by another element at its click point".to_string(),
            ));
        }

        self.page.clone().click_mouse(target_x, target_y).await?;

        Ok(self)
    }

    /// Hovers over the element using realistic mouse movement.
    pub async fn hover_mouse(self: Arc<Self>) -> XcelerateResult<Arc<Self>> {
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
        check_exception(res.exception_details)?;
        Ok(res
            .result
            .value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string()))
    }

    /// Calls a function on this element and coerces the result to a string.
    pub async fn evaluate_string(&self, function: String) -> XcelerateResult<String> {
        let res = self.call_js(function).await?;
        check_exception(res.exception_details)?;
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
        check_exception(res.exception_details)?;
        Ok(res.result.value.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// Focuses the element and presses a key or a chord.
    ///
    /// `key` is a single key (`Enter`, `a`) or a chord joined with `+`
    /// (`ctrl+a`, `ctrl+shift+k`). Modifiers are `ctrl`, `shift`, `alt`, `meta`.
    pub async fn press(self: Arc<Self>, key: String) -> XcelerateResult<()> {
        self.clone().focus().await?;
        let parts: Vec<&str> = key
            .split('+')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect();
        let Some((main, modifier_names)) = parts.split_last() else {
            return Err(XcelerateError::Unsupported("empty key".to_string()));
        };
        let main = *main;

        // Resolve the modifiers and their combined CDP bitmask.
        let mut bits = 0i64;
        let mut modifiers = Vec::new();
        for name in modifier_names {
            let (vk, code, label, bit) = modifier_info(name).ok_or_else(|| {
                XcelerateError::Unsupported(format!("unknown modifier `{name}` in `{key}`"))
            })?;
            bits |= bit;
            modifiers.push((vk, code, label, bit));
        }

        // Modifier keydowns (accumulating the bitmask), the key itself, then the
        // matching keyups in reverse.
        let mut active = 0i64;
        for (vk, code, label, bit) in &modifiers {
            active |= bit;
            self.dispatch_key("keyDown", active, *vk, code, label, None)
                .await?;
        }
        let (virtual_key, code, name) = key_info(main);
        let text = if main.chars().count() == 1 {
            Some(main.to_string())
        } else {
            None
        };
        self.dispatch_key("keyDown", bits, virtual_key, &code, &name, text.as_deref())
            .await?;
        self.dispatch_key("keyUp", bits, virtual_key, &code, &name, None)
            .await?;
        for (vk, code, label, bit) in modifiers.iter().rev() {
            active &= !bit;
            self.dispatch_key("keyUp", active, *vk, code, label, None)
                .await?;
        }
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
        check_exception(res.exception_details)?;
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
    ///
    /// The search pierces open shadow roots, so web components are reachable.
    pub async fn query_selector(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Arc<Element>> {
        match self
            .call_with_args(JS_QUERY_ONE, vec![serde_json::json!(selector)], false)
            .await?
        {
            Some(object_id) => Ok(Arc::new(Element {
                page: self.page.clone(),
                object_id,
            })),
            None => Err(XcelerateError::NotFound(selector)),
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
        check_exception(res.exception_details)?;
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
    /// The search pierces open shadow roots. Resolves the whole node list with a
    /// single `Runtime.getProperties` call rather than one `evaluate` per match.
    pub async fn query_selector_all(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Vec<Arc<Element>>> {
        match self
            .call_with_args(JS_QUERY_ALL, vec![serde_json::json!(selector)], false)
            .await?
        {
            Some(object_id) => {
                crate::page::collect_elements(
                    &self.page.client,
                    &self.page.session_id,
                    &self.page,
                    object_id.into(),
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
    ///
    /// The descendant is resolved with the shadow-piercing selector first, so
    /// the expression also runs against a match inside an open shadow root.
    pub async fn call_on_selector(
        &self,
        selector: String,
        expression: String,
    ) -> XcelerateResult<String> {
        match self
            .call_with_args(JS_QUERY_ONE, vec![serde_json::json!(selector)], false)
            .await?
        {
            Some(object_id) => {
                let element = Element {
                    page: self.page.clone(),
                    object_id,
                };
                element
                    .call_json(
                        "function(src){return (new Function('el','return ('+src+')(el);'))(this);}"
                            .to_string(),
                        serde_json::json!([expression]).to_string(),
                    )
                    .await
            }
            None => Ok("null".to_string()),
        }
    }

    /// Runs a JS function against every descendant matching `selector`.
    ///
    /// The descendants are resolved with the shadow-piercing selector first, so
    /// matches inside open shadow roots are included too.
    pub async fn call_on_selector_all(
        &self,
        selector: String,
        expression: String,
    ) -> XcelerateResult<String> {
        match self
            .call_with_args(JS_QUERY_ALL, vec![serde_json::json!(selector)], false)
            .await?
        {
            Some(object_id) => {
                let elements = Element {
                    page: self.page.clone(),
                    object_id,
                };
                elements
                    .call_json(
                        "function(src){const fn=new Function('el','return ('+src+')(el);');return Array.from(this).map(fn);}"
                            .to_string(),
                        serde_json::json!([expression]).to_string(),
                    )
                    .await
            }
            None => Ok("[]".to_string()),
        }
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
    ///
    /// The search pierces open shadow roots.
    pub async fn get_by_text(self: Arc<Self>, text: String) -> XcelerateResult<Arc<Element>> {
        match self
            .call_with_args(JS_BY_TEXT, vec![serde_json::json!(text)], false)
            .await?
        {
            Some(object_id) => Ok(Arc::new(Element {
                page: self.page.clone(),
                object_id,
            })),
            None => Err(XcelerateError::NotFound(format!("text not found: {text}"))),
        }
    }

    /// Finds a descendant by ARIA role.
    ///
    /// Prefers an explicit `[role="..."]` match, then falls back to the role name
    /// as a tag, since a native `<button>`/`<a>` carries its role implicitly.
    /// Both searches pierce open shadow roots.
    pub async fn get_by_role(self: Arc<Self>, role: String) -> XcelerateResult<Arc<Element>> {
        match self
            .clone()
            .query_selector_attr("role".to_string(), role.clone())
            .await
        {
            Ok(element) => Ok(element),
            Err(_) => self.query_selector(role).await,
        }
    }

    /// Finds a descendant form control by its `<label>` text.
    ///
    /// The `<label>` search pierces open shadow roots, and the associated control
    /// is resolved from the label's own root so shadow-encapsulated controls work.
    pub async fn get_by_label(self: Arc<Self>, label: String) -> XcelerateResult<Arc<Element>> {
        let labels = match self
            .call_with_args(JS_QUERY_ALL, vec![serde_json::json!("label")], false)
            .await?
        {
            Some(object_id) => Element {
                page: self.page.clone(),
                object_id,
            },
            None => {
                return Err(XcelerateError::NotFound(format!(
                    "label not found: {label}"
                )));
            }
        };
        let js = "function(label){const labels=Array.from(this);for(let i=0;i<labels.length;i++){const lab=labels[i];if(!(lab.textContent||'').trim().includes(label))continue;if(lab.control)return lab.control;const f=lab.getAttribute('for');if(!f)continue;const root=lab.getRootNode();const c=root&&root.getElementById?root.getElementById(f):null;if(c)return c;}return null;}";
        match labels
            .call_with_args(js, vec![serde_json::json!(label)], false)
            .await?
        {
            Some(object_id) => Ok(Arc::new(Element {
                page: self.page.clone(),
                object_id,
            })),
            None => Err(XcelerateError::NotFound(format!(
                "label not found: {label}"
            ))),
        }
    }

    /// Finds a descendant matching an XPath expression.
    ///
    /// The expression is evaluated over the composed tree - open shadow roots and
    /// same-origin iframe documents are searched - by a built-in subset evaluator.
    /// Expressions outside that subset (unions, extra axes, `count()`, ...) fall
    /// back to the browser's native `document.evaluate`, which handles the full
    /// language but does not pierce shadow roots.
    pub async fn query_selector_xpath(
        self: Arc<Self>,
        xpath: String,
    ) -> XcelerateResult<Arc<Element>> {
        let args = vec![serde_json::json!(xpath), serde_json::json!(false)];
        if let Ok(Some(object_id)) = self.call_with_args(JS_XPATH, args, false).await {
            return Ok(Arc::new(Element {
                page: self.page.clone(),
                object_id,
            }));
        }

        // Native fallback: full XPath support, no shadow piercing. Evaluated
        // through `call_with_args` so an invalid expression is reported, not
        // mistaken for a match.
        let args = vec![serde_json::json!(xpath)];
        match self.call_with_args(JS_XPATH_NATIVE, args, false).await? {
            Some(object_id) => Ok(Arc::new(Element {
                page: self.page.clone(),
                object_id,
            })),
            None => Err(XcelerateError::NotFound(xpath)),
        }
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
    ///
    /// The wait happens inside the page in a single CDP call: a `MutationObserver`
    /// resolves as soon as the node appears (and a slow rescan covers shadow
    /// roots), instead of the caller polling `query_selector` over the wire every
    /// 250ms. Times out after 30 seconds.
    pub async fn wait_for_selector(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Arc<Element>> {
        let args = vec![serde_json::json!(selector), serde_json::json!(30_000u64)];
        match self
            .call_with_args(JS_WAIT_FOR_SELECTOR, args, true)
            .await?
        {
            Some(object_id) => Ok(Arc::new(Element {
                page: self.page.clone(),
                object_id,
            })),
            None => Err(XcelerateError::NotFound(format!(
                "Timeout waiting for selector: {selector}"
            ))),
        }
    }
}

impl Element {
    /// Runs the action-point probe. `random` jitters the point inside the element
    /// (clicks) rather than using the centre (drags). Returns the viewport point
    /// plus whether the element is visible and actually on top there.
    async fn probe_action_point(&self, random: bool) -> XcelerateResult<(f64, f64, bool, bool)> {
        let json = self
            .call_json(
                JS_ACTION_POINT.to_string(),
                if random { "[true]" } else { "[false]" }.to_string(),
            )
            .await?;
        let point: ActionPoint =
            serde_json::from_str(&json).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        Ok((point.x, point.y, point.visible, point.on_top))
    }

    /// The viewport point a real pointer action (click/drag) should use for this
    /// element: its centre, translated out of any same-origin frames.
    ///
    /// Errors when the element is not visible or is covered by another element at
    /// that point, so a caller never reports a click that would hit nothing.
    /// Kept out of the `#[uniffi::export]` block so binding checksums stay stable.
    pub async fn action_point(&self) -> XcelerateResult<(f64, f64)> {
        let (x, y, visible, on_top) = self.probe_action_point(false).await?;
        if !visible {
            return Err(XcelerateError::NotFound(
                "element is not visible (zero-size, hidden or transparent)".to_string(),
            ));
        }
        if !on_top {
            return Err(XcelerateError::NotFound(
                "element is covered by another element at its action point".to_string(),
            ));
        }
        Ok((x, y))
    }

    /// Dispatches one CDP key event with an explicit modifier bitmask.
    async fn dispatch_key(
        &self,
        type_: &str,
        modifiers: i64,
        virtual_key: i64,
        code: &str,
        key: &str,
        text: Option<&str>,
    ) -> XcelerateResult<()> {
        let mut params = DispatchKeyEventParams {
            type_: type_.into(),
            ..Default::default()
        };
        params.modifiers = Some(modifiers);
        params.key = Some(key.to_string().into());
        params.code = Some(code.to_string().into());
        params.windows_virtual_key_code = Some(virtual_key);
        params.native_virtual_key_code = Some(virtual_key);
        if let Some(text) = text {
            params.text = Some(text.to_string().into());
            params.unmodified_text = Some(text.to_string().into());
        }
        self.page
            .client
            .execute_with_session(Some(&self.page.session_id), params)
            .await?;
        Ok(())
    }

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

    /// Calls `js` on this element with the given JSON arguments and returns the
    /// resulting remote object id, if the call produced one.
    ///
    /// `await_promise` resolves a promise the function returns (used by the
    /// in-page selector wait).
    async fn call_with_args(
        &self,
        js: &str,
        args: Vec<serde_json::Value>,
        await_promise: bool,
    ) -> XcelerateResult<Option<String>> {
        let arguments: Vec<js_protocol::runtime::CallArgument<'_>> = args
            .into_iter()
            .map(|value| js_protocol::runtime::CallArgument {
                value: Some(value),
                ..Default::default()
            })
            .collect();
        let res = self
            .page
            .client
            .execute_with_session(
                Some(&self.page.session_id),
                js_protocol::runtime::CallFunctionOnParams {
                    function_declaration: js.into(),
                    object_id: Some(self.object_id.clone().into()),
                    arguments: Some(arguments),
                    await_promise: Some(await_promise),
                    return_by_value: Some(false),
                    ..Default::default()
                },
            )
            .await?;
        // A thrown function is not a result; surface the thrown message (the
        // XPath layer uses this to fall back to native `document.evaluate`).
        check_exception(res.exception_details)?;
        Ok(res.result.object_id.map(|id| id.into_owned()))
    }

    /// Returns every descendant matching `selector`, piercing open shadow roots
    /// **and same-origin iframe documents**.
    ///
    /// Unlike [`Element::query_selector_all`], this crosses frame boundaries, so
    /// it reaches content inside a same-origin `<iframe>`. Cross-origin frames
    /// cannot be reached from page script and are skipped. Kept out of the
    /// `#[uniffi::export]` block so binding checksums stay stable.
    pub async fn query_selector_all_frames(
        self: Arc<Self>,
        selector: String,
    ) -> XcelerateResult<Vec<Arc<Element>>> {
        match self
            .call_with_args(
                JS_QUERY_ALL_FRAMES,
                vec![serde_json::json!(selector)],
                false,
            )
            .await?
        {
            Some(object_id) => {
                crate::page::collect_elements(
                    &self.page.client,
                    &self.page.session_id,
                    &self.page,
                    object_id.into(),
                )
                .await
            }
            None => Ok(Vec::new()),
        }
    }

    /// Returns every descendant matching an XPath expression.
    ///
    /// Like [`Element::query_selector_xpath`], the search pierces open shadow
    /// roots and same-origin iframe documents via the built-in subset evaluator;
    /// expressions outside that subset fall back to native `document.evaluate`
    /// (full language, no shadow piercing). Kept out of the
    /// `#[uniffi::export]` block so binding checksums stay stable.
    pub async fn query_selector_all_xpath(
        self: Arc<Self>,
        xpath: String,
    ) -> XcelerateResult<Vec<Arc<Element>>> {
        let args = vec![serde_json::json!(xpath), serde_json::json!(true)];
        if let Ok(Some(object_id)) = self.call_with_args(JS_XPATH, args, false).await {
            return crate::page::collect_elements(
                &self.page.client,
                &self.page.session_id,
                &self.page,
                object_id.into(),
            )
            .await;
        }

        // Native fallback: full XPath support, no shadow piercing.
        let args = vec![serde_json::json!(xpath)];
        match self
            .call_with_args(JS_XPATH_ALL_NATIVE, args, false)
            .await?
        {
            Some(object_id) => {
                crate::page::collect_elements(
                    &self.page.client,
                    &self.page.session_id,
                    &self.page,
                    object_id.into(),
                )
                .await
            }
            None => Ok(Vec::new()),
        }
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

/// Maps a modifier name to `(virtual key, code, key, CDP modifier bit)`.
///
/// The CDP bits are Alt = 1, Ctrl = 2, Meta = 4, Shift = 8.
fn modifier_info(name: &str) -> Option<(i64, String, String, i64)> {
    Some(match name.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => (17, "ControlLeft".to_string(), "Control".to_string(), 2),
        "shift" => (16, "ShiftLeft".to_string(), "Shift".to_string(), 8),
        "alt" => (18, "AltLeft".to_string(), "Alt".to_string(), 1),
        "meta" | "cmd" | "super" | "win" => (91, "MetaLeft".to_string(), "Meta".to_string(), 4),
        _ => return None,
    })
}

/// Extracts a human-readable message from CDP `exceptionDetails`.
///
/// Prefers the thrown object's `description` (the `Error` message and stack)
/// and falls back to the protocol `text` when no exception object was reported.
fn exception_message(exception: js_protocol::runtime::ExceptionDetails<'_>) -> String {
    exception
        .exception
        .and_then(|exception| exception.description)
        .map(std::borrow::Cow::into_owned)
        .unwrap_or_else(|| exception.text.into_owned())
}

/// Turns a `Runtime.evaluate` / `Runtime.callFunctionOn` `exceptionDetails` into
/// an error, so a thrown expression is never mistaken for a result.
///
/// Both commands report a throw in-band: `result` still carries the thrown
/// object's handle, so without this a `throw` would be returned as a value.
/// `null`/`undefined` results have no `exceptionDetails` and are unaffected.
pub(crate) fn check_exception(
    exception: Option<js_protocol::runtime::ExceptionDetails<'_>>,
) -> XcelerateResult<()> {
    match exception {
        Some(exception) => Err(XcelerateError::Unsupported(exception_message(exception))),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use js_protocol::runtime::{ExceptionDetails, RemoteObject};

    fn details(description: Option<&str>, text: &str) -> ExceptionDetails<'static> {
        ExceptionDetails {
            text: text.to_string().into(),
            exception: description.map(|description| RemoteObject {
                description: Some(description.to_string().into()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn exception_message_prefers_description() {
        assert_eq!(exception_message(details(Some("boom"), "Uncaught")), "boom");
    }

    #[test]
    fn exception_message_falls_back_to_text() {
        assert_eq!(exception_message(details(None, "Uncaught")), "Uncaught");
    }

    #[test]
    fn exception_message_falls_back_when_description_is_absent() {
        let exception = ExceptionDetails {
            text: "SyntaxError".into(),
            exception: Some(RemoteObject {
                type_: "object".into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(exception_message(exception), "SyntaxError");
    }

    #[test]
    fn check_exception_is_ok_without_details() {
        assert!(check_exception(None).is_ok());
    }

    #[test]
    fn check_exception_reports_the_thrown_message() {
        let error = check_exception(Some(details(Some("boom"), "Uncaught"))).unwrap_err();
        assert!(matches!(error, XcelerateError::Unsupported(message) if message == "boom"));
    }
}
