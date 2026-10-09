//! Storage verbs: read/write `localStorage` and `sessionStorage` through the
//! page's JS bridge, and save/restore the full storage state (cookies +
//! localStorage) through the core `storage_state` / `set_storage_state`.
//!
//! Storage entries live in the page, so they go through `Page::evaluate_json`
//! (the existing eval CDP path); the full-state save/restore uses the core
//! methods, which also round-trip cookies.

use xcelerate_interpreter::runtime::resolve_in_root;

use super::state::Session;

impl Session {
    /// `storage <local|session> [get [key]|set <key> <value>|clear]` /
    /// `storage save <path>` / `storage restore <path>`.
    pub(crate) async fn storage(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sub = tokens
            .get(1)
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        match sub.as_str() {
            "local" | "session" => {
                let store = if sub == "local" {
                    "localStorage"
                } else {
                    "sessionStorage"
                };
                let action = tokens
                    .get(2)
                    .map(|s| s.to_ascii_lowercase())
                    .unwrap_or_default();
                match action.as_str() {
                    // No action, or `get`/`list`: dump the whole store as JSON.
                    // A key narrows it to that one entry.
                    "" | "get" | "list" => match tokens.get(3) {
                        Some(key) if !key.is_empty() => {
                            let script = format!("{store}.getItem({})", js_literal(key));
                            let raw = self.page.evaluate_json(script).await?;
                            match serde_json::from_str::<serde_json::Value>(&raw) {
                                Ok(serde_json::Value::String(text)) => println!("{text}"),
                                _ => println!("{raw}"),
                            }
                        }
                        _ => {
                            let script = format!("Object.fromEntries(Object.entries({store}))");
                            println!("{}", self.page.evaluate_json(script).await?);
                        }
                    },
                    "set" => {
                        let key = tokens.get(3).cloned().unwrap_or_default();
                        let value = tokens.get(4).cloned().unwrap_or_default();
                        if key.is_empty() {
                            println!("usage: storage {sub} set <key> <value>");
                            return Ok(());
                        }
                        let script = format!(
                            "{store}.setItem({}, {})",
                            js_literal(&key),
                            js_literal(&value)
                        );
                        self.page.evaluate_json(script).await?;
                        println!("storage {sub} set {key}");
                    }
                    "clear" => {
                        self.page.evaluate_json(format!("{store}.clear()")).await?;
                        println!("storage {sub} cleared");
                    }
                    other => println!(
                        "usage: storage {sub} [get [key]|set <key> <value>|clear] (got `{other}`)"
                    ),
                }
            }
            "save" => {
                let raw = tokens.get(2).cloned().unwrap_or_default();
                if raw.is_empty() {
                    println!("usage: storage save <path>");
                    return Ok(());
                }
                let path = resolve_in_root(&self.root, &raw)?;
                let state = self.page.storage_state().await?;
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, &state)?;
                println!(
                    "wrote storage state to {} ({} bytes)",
                    path.display(),
                    state.len()
                );
            }
            "restore" => {
                let raw = tokens.get(2).cloned().unwrap_or_default();
                if raw.is_empty() {
                    println!("usage: storage restore <path>");
                    return Ok(());
                }
                let path = resolve_in_root(&self.root, &raw)?;
                let json = std::fs::read_to_string(&path)?;
                self.page.set_storage_state(json).await?;
                println!("restored storage state from {}", path.display());
            }
            _ => println!(
                "usage: storage <local|session> [get [key]|set <key> <value>|clear] | \
                 storage save <path> | storage restore <path>"
            ),
        }
        Ok(())
    }
}

/// Renders a Rust string as a JavaScript string literal, so a key or value that
/// contains quotes, backslashes or newlines cannot break out of the expression.
fn js_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}
