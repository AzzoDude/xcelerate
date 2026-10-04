//! Built-in `human` plugin.
//!
//! Turns automation input into something that behaves like a person: mouse
//! travel follows a jittered Bezier path, clicks pause before pressing and hold
//! before releasing, typing has per-key delays, and scrolling happens in uneven
//! steps. The plugin drives the page only through the [`ArcPageHost`] interface.

use std::sync::{Arc, Mutex};

use xcelerate_plugin_api::{
    ArcPageHost, BoxFut, Budgets, Capability, Manifest, OpCall, Plugin, PluginError, PluginResult,
    Registry,
};

/// Human-like input for the focused page.
#[derive(Default)]
pub struct HumanPlugin {
    pages: Arc<Mutex<Vec<ArcPageHost>>>,
}

/// Small, non-cryptographic jitter source; only used for timing and space noise.
struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn range(&mut self, min: f64, max: f64) -> f64 {
        let unit = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        min + unit * (max - min)
    }

    fn millis(&mut self, min: u64, max: u64) -> u64 {
        self.range(min as f64, max as f64) as u64
    }
}

fn latest(pages: &Arc<Mutex<Vec<ArcPageHost>>>) -> PluginResult<ArcPageHost> {
    pages
        .lock()
        .unwrap()
        .last()
        .cloned()
        .ok_or_else(|| PluginError::NotFound("human: no page has been created yet".to_string()))
}

fn args(call: &OpCall) -> serde_json::Value {
    serde_json::from_str(&call.args_json).unwrap_or_else(|_| serde_json::json!({}))
}

fn number(value: &serde_json::Value, key: &str, op: &str) -> PluginResult<f64> {
    value
        .get(key)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| PluginError::Unsupported(format!("human.{op}: numeric '{key}' is required")))
}

fn string_arg(value: &serde_json::Value, key: &str, op: &str) -> PluginResult<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| PluginError::Unsupported(format!("human.{op}: string '{key}' is required")))
}

impl Plugin for HumanPlugin {
    fn name(&self) -> &str {
        "human"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            name: "human".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            host_api: "1.x".to_string(),
            entrypoint: None,
            abi: Some("native/1".to_string()),
            ops: ["info", "move", "click", "type", "scroll", "delay"]
                .iter()
                .map(|op| (*op).to_string())
                .collect(),
            capabilities: vec![Capability::Click, Capability::TypeKeys, Capability::Query],
            dependencies: Default::default(),
            limits: Budgets::default(),
        }
    }

    fn build(&self, reg: &mut Registry) {
        reg.op("info", |_call| {
            Box::pin(async {
                Ok(serde_json::json!({
                    "name": "human",
                    "enabled": true,
                    "ops": ["move", "click", "type", "scroll", "delay"],
                })
                .to_string())
            })
        });

        let pages = Arc::clone(&self.pages);
        reg.op("move", move |call| {
            let pages = Arc::clone(&pages);
            Box::pin(async move {
                let value = args(&call);
                let x = number(&value, "x", "move")?;
                let y = number(&value, "y", "move")?;
                latest(&pages)?.move_mouse(x, y).await?;
                Ok(serde_json::json!({ "moved": true, "x": x, "y": y }).to_string())
            })
        });

        let pages = Arc::clone(&self.pages);
        reg.op("click", move |call| {
            let pages = Arc::clone(&pages);
            Box::pin(async move {
                let value = args(&call);
                let x = number(&value, "x", "click")?;
                let y = number(&value, "y", "click")?;
                latest(&pages)?.click_mouse(x, y).await?;
                Ok(serde_json::json!({ "clicked": true, "x": x, "y": y }).to_string())
            })
        });

        let pages = Arc::clone(&self.pages);
        reg.op("type", move |call| {
            let pages = Arc::clone(&pages);
            Box::pin(async move {
                let value = args(&call);
                let text = string_arg(&value, "text", "type")?;
                let page = latest(&pages)?;
                let mut rng = Rng::new();
                for character in text.chars() {
                    page.keyboard_type(character.to_string()).await?;
                    let delay = rng.millis(30, 140);
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
                Ok(serde_json::json!({ "typed": text.chars().count() }).to_string())
            })
        });

        let pages = Arc::clone(&self.pages);
        reg.op("scroll", move |call| {
            let pages = Arc::clone(&pages);
            Box::pin(async move {
                let value = args(&call);
                let delta = number(&value, "deltaY", "scroll")?;
                let page = latest(&pages)?;
                let (x, y) = page.mouse_position();
                page.dispatch_cdp(
                    "Input.dispatchMouseEvent".to_string(),
                    serde_json::json!({
                        "type": "mouseWheel",
                        "x": x,
                        "y": y,
                        "deltaX": 0.0,
                        "deltaY": delta,
                    })
                    .to_string(),
                )
                .await?;
                let mut rng = Rng::new();
                let delay = rng.millis(40, 160);
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                Ok(serde_json::json!({ "scrolled": delta }).to_string())
            })
        });

        reg.op("delay", |call| {
            Box::pin(async move {
                let value = args(&call);
                let min = value
                    .get("minMs")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(120);
                let max = value
                    .get("maxMs")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(min.max(400));
                let mut rng = Rng::new();
                let delay = rng.millis(min.min(max), min.max(max));
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                Ok(serde_json::json!({ "sleptMs": delay }).to_string())
            })
        });
    }

    fn on_page_created(&self, page: ArcPageHost) -> BoxFut<PluginResult<()>> {
        let pages = Arc::clone(&self.pages);
        Box::pin(async move {
            pages.lock().unwrap().push(page);
            Ok(())
        })
    }
}
