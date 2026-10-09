//! Tab management: new-tab, switch, tabs and close-tab.

use std::sync::Arc;

use super::input::focus_page;
use super::state::Session;

impl Session {
    /// `new-tab [url]`: open a new tab and make it active.
    pub(crate) async fn new_tab(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = if rest.is_empty() {
            "about:blank".to_string()
        } else {
            rest.to_string()
        };
        let opened = Arc::clone(&self.browser).new_page(url).await?;
        opened.set_human(!self.args.linear);
        if self.args.cursor_active() {
            let _ = crate::cursor::install(&opened).await;
            crate::cursor::set_gate(&opened, true).await;
        }
        self.tabs.push(Arc::clone(&opened));
        self.active_tab = self.tabs.len() - 1;
        self.page = opened;
        // Bring the new tab (and its window) to the front.
        focus_page(&self.page).await;
        println!(
            "opened tab {} -> {}",
            self.active_tab,
            self.page.url().await.unwrap_or_default()
        );
        Ok(())
    }

    /// `switch <index|targetId>`: make a tab active.
    ///
    /// An exact target-id match wins over index parsing, so an all-numeric id is
    /// never mistaken for an index. A bare number that matches no open tab is
    /// read as an index.
    pub(crate) async fn switch(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if rest.is_empty() {
            // No argument cycles to the next tab.
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
            self.page = self.tabs[self.active_tab].clone();
        } else if let Some(position) = self.tabs.iter().position(|tab| tab.target_id() == rest) {
            // A target id this session already knows about: exact match first.
            self.active_tab = position;
            self.page = self.tabs[position].clone();
        } else if let Ok(index) = rest.parse::<usize>() {
            if index < self.tabs.len() {
                self.active_tab = index;
                self.page = self.tabs[index].clone();
            } else {
                println!("no tab {index} (have {} open)", self.tabs.len());
                return Ok(());
            }
        } else {
            // A target this session did not open (for example a popup).
            match Arc::clone(&self.browser)
                .attach_page(rest.to_string())
                .await
            {
                Ok(attached) => {
                    attached.set_human(!self.args.linear);
                    if self.args.cursor_active() {
                        let _ = crate::cursor::install(&attached).await;
                        crate::cursor::set_gate(&attached, true).await;
                    }
                    self.tabs.push(Arc::clone(&attached));
                    self.active_tab = self.tabs.len() - 1;
                    self.page = attached;
                }
                Err(error) => {
                    println!("could not switch to {rest}: {error}");
                    return Ok(());
                }
            }
        }
        println!(
            "switched to tab {} -> {}",
            self.active_tab,
            self.page.url().await.unwrap_or_default()
        );
        // Bring the tab (and its window) to the front.
        focus_page(&self.page).await;
        Ok(())
    }

    /// `tabs`: list targets (id, type, url).
    pub(crate) async fn tabs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let raw = self.browser.targets().await?;
        let parsed: serde_json::Value =
            serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
        // `Target.getTargets` wraps the array in `targetInfos`.
        let infos = parsed
            .get("targetInfos")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .or_else(|| parsed.as_array().cloned());
        let Some(items) = infos else {
            println!("{raw}");
            return Ok(());
        };

        // The tabs this session opened, in the order `switch` /
        // `close-tab` index them. The active one is marked `*`.
        let session_ids: Vec<String> = self
            .tabs
            .iter()
            .map(|tab| tab.target_id().to_string())
            .collect();
        for (index, tab) in self.tabs.iter().enumerate() {
            let mark = if index == self.active_tab { "*" } else { " " };
            let url = tab.url().await.unwrap_or_default();
            println!("{mark} [{index}]  {}  {url}", tab.target_id());
        }
        // Everything else the browser reports (extensions, workers,
        // popups this session did not open): no index, id only.
        for item in items {
            let id = item
                .get("targetId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if session_ids.iter().any(|known| known == id) {
                continue;
            }
            let kind = item
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let url = item
                .get("url")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            println!("  -   {id}  {kind}  {url}");
        }
        Ok(())
    }

    /// `close-tab <index|targetId>`.
    ///
    /// An exact target-id match wins over index parsing, so an all-numeric id is
    /// never mistaken for an index. A bare number that matches no open tab is
    /// read as an index.
    pub(crate) async fn close_tab(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if rest.is_empty() {
            println!("usage: close-tab <index|targetId>   (indices/ids come from `tabs`)");
            return Ok(());
        }
        // Accept a raw id (exact match first), or the same session index
        // `switch` uses.
        let target_id =
            if let Some(position) = self.tabs.iter().position(|tab| tab.target_id() == rest) {
                self.tabs[position].target_id()
            } else if let Ok(index) = rest.parse::<usize>() {
                match self.tabs.get(index) {
                    Some(tab) => tab.target_id(),
                    None => {
                        println!("no tab {index} (have {} open)", self.tabs.len());
                        return Ok(());
                    }
                }
            } else {
                rest.to_string()
            };
        // Never close the session's last tab: that would leave the
        // session with nothing to drive.
        if self.tabs.len() <= 1
            && self
                .tabs
                .iter()
                .any(|tab| tab.target_id() == target_id.as_str())
        {
            println!("cannot close the last tab");
            return Ok(());
        }
        let params = serde_json::json!({ "targetId": target_id }).to_string();
        match self
            .page
            .execute_cdp_cmd("Target.closeTarget".to_string(), params)
            .await
        {
            Ok(_) => {
                println!("closed tab {target_id}");
                // Drop it from the list, keeping the active tab valid.
                if let Some(position) = self
                    .tabs
                    .iter()
                    .position(|tab| tab.target_id() == target_id.as_str())
                {
                    self.tabs.remove(position);
                    if self.active_tab >= self.tabs.len() {
                        self.active_tab = self.tabs.len().saturating_sub(1);
                    }
                    if !self.tabs.is_empty() {
                        self.page = self.tabs[self.active_tab].clone();
                        focus_page(&self.page).await;
                    }
                }
            }
            Err(error) => println!("could not close {target_id}: {error}"),
        }
        Ok(())
    }
}
