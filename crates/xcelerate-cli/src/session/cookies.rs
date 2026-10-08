//! Cookie verbs: read, set, and clear the browser's cookies.
//!
//! Cookies go through CDP (`Network.setCookie`), not page JS, so `HttpOnly`
//! session cookies - which `document.cookie` cannot write - can be restored.

use super::state::Session;

impl Session {
    /// `cookie [get] [name]`, `cookie set <name> <value> [domain] [path]`,
    /// `cookie add <json>`, `cookie delete <name>`, `cookie clear`.
    pub(crate) async fn cookie(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sub = tokens
            .get(1)
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        match sub.as_str() {
            // No subcommand, or `get`/`list`: dump cookies as JSON. A name
            // argument narrows it to that one cookie.
            "" | "get" | "list" => match tokens.get(2) {
                Some(name) if !name.is_empty() => {
                    println!("{}", self.page.cookie(name.clone()).await?);
                }
                _ => println!("{}", self.page.cookies().await?),
            },
            "set" => {
                let name = tokens.get(2).cloned().unwrap_or_default();
                let value = tokens.get(3).cloned().unwrap_or_default();
                if name.is_empty() {
                    println!("usage: cookie set <name> <value> [domain] [path]");
                    return Ok(());
                }
                let mut cookie = serde_json::json!({ "name": name, "value": value, "path": "/" });
                match tokens.get(4).filter(|d| !d.is_empty()) {
                    Some(domain) => {
                        cookie["domain"] = serde_json::Value::String(domain.clone());
                        if let Some(path) = tokens.get(5).filter(|p| !p.is_empty()) {
                            cookie["path"] = serde_json::Value::String(path.clone());
                        }
                    }
                    // `Network.setCookie` needs a `url` or a `domain`; fall back
                    // to the page's own URL so `cookie set` works on the open page.
                    None => {
                        cookie["url"] =
                            serde_json::Value::String(self.page.url().await.unwrap_or_default());
                    }
                }
                self.page
                    .execute_cdp_cmd("Network.setCookie".to_string(), cookie.to_string())
                    .await?;
                println!("cookie set {name}");
            }
            "add" | "import" => {
                let json = tokens.get(2).cloned().unwrap_or_default();
                if json.is_empty() {
                    println!("usage: cookie add <json>");
                    return Ok(());
                }
                let parsed: serde_json::Value = serde_json::from_str(&json)
                    .map_err(|e| format!("cookie add: invalid JSON: {e}"))?;
                let list = match parsed {
                    serde_json::Value::Array(list) => list,
                    other => vec![other],
                };
                for cookie in &list {
                    self.page
                        .execute_cdp_cmd("Network.setCookie".to_string(), cookie.to_string())
                        .await?;
                }
                println!("added {} cookie(s)", list.len());
            }
            "delete" | "remove" | "rm" => {
                let name = tokens.get(2).cloned().unwrap_or_default();
                if name.is_empty() {
                    println!("usage: cookie delete <name>");
                    return Ok(());
                }
                self.page
                    .execute_cdp_cmd(
                        "Network.deleteCookies".to_string(),
                        serde_json::json!({ "name": name }).to_string(),
                    )
                    .await?;
                println!("cookie delete {name}");
            }
            "clear" | "clear-all" => {
                self.page
                    .execute_cdp_cmd("Network.clearBrowserCookies".to_string(), "{}".to_string())
                    .await?;
                println!("cookies cleared");
            }
            other => println!(
                "usage: cookie [get [name]|set <name> <value> [domain] [path]|add <json>|delete <name>|clear] (got `{other}`)"
            ),
        }
        Ok(())
    }
}
