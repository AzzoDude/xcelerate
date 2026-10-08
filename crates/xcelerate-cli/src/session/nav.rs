//! Verbs that navigate the page or read from it.

use std::time::Duration;

use super::help::print_snapshot;
use super::state::Session;

impl Session {
    /// `open <url>` / `goto <url>`: navigate the live page.
    pub(crate) async fn open(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if rest.is_empty() {
            println!("usage: open <url>");
            return Ok(());
        }
        self.page
            .navigate(xcelerate_interpreter::runtime::normalize_url(rest))
            .await?;
        let _ = self.page.wait_for_navigation().await;
        let _ = self.page.wait_for_dom_stable(300, 2_000).await;
        println!("title: {}", self.page.title().await.unwrap_or_default());
        println!("url:   {}", self.page.url().await.unwrap_or_default());
        Ok(())
    }

    /// `title`.
    pub(crate) async fn title(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.page.title().await?);
        Ok(())
    }

    /// `url`.
    pub(crate) async fn url(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.page.url().await?);
        Ok(())
    }

    /// `text`: the page's visible text.
    pub(crate) async fn text(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let text = self
            .page
            .evaluate_string("document.body ? document.body.innerText : ''".to_string())
            .await?;
        println!("{text}");
        Ok(())
    }

    /// `snapshot` / `snap`: indexed interactive elements.
    pub(crate) async fn snapshot(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // `snapshot_json` also refreshes the index map `click` reads.
        let json = self.page.snapshot_json().await?;
        print_snapshot(&json);
        Ok(())
    }

    /// `back`.
    pub(crate) async fn back(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.page.go_back().await?;
        let _ = self.page.wait_for_navigation().await;
        println!("{}", self.page.url().await.unwrap_or_default());
        Ok(())
    }

    /// `reload`.
    pub(crate) async fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.page.reload().await?;
        let _ = self.page.wait_for_navigation().await;
        Ok(())
    }

    /// `scroll <pixels|up|down|top|bottom>`.
    pub(crate) async fn scroll(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        let js = match rest {
            "" | "down" => "window.scrollBy(0, window.innerHeight * 0.9)".to_string(),
            "up" => "window.scrollBy(0, -window.innerHeight * 0.9)".to_string(),
            "top" => "window.scrollTo(0, 0)".to_string(),
            "bottom" => "window.scrollTo(0, document.body.scrollHeight)".to_string(),
            other => match other.parse::<i64>() {
                Ok(pixels) => format!("window.scrollBy(0, {pixels})"),
                Err(_) => {
                    println!("usage: scroll <pixels|up|down|top|bottom>");
                    String::new()
                }
            },
        };
        if !js.is_empty() {
            let _ = self.page.evaluate_string(js).await;
            println!("scrolled {rest}");
        }
        Ok(())
    }

    /// `markdown` / `md`: the main content as Markdown.
    pub(crate) async fn markdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.page.markdown().await?);
        Ok(())
    }

    /// `content` / `html`: the raw HTML.
    pub(crate) async fn content(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.page.content().await?);
        Ok(())
    }

    /// `find <text>`: how many elements contain the text.
    pub(crate) async fn find(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        let count = self.page.find_text(rest.to_string()).await?;
        println!("{count} match(es) for {rest:?}");
        Ok(())
    }

    /// `eval <js>` / `js`: evaluate JavaScript, print the result.
    pub(crate) async fn eval(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        let raw = self.page.evaluate_json(rest.to_string()).await?;
        // Unwrap a top-level string so `eval document.title` reads cleanly.
        match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(serde_json::Value::String(text)) => println!("{text}"),
            _ => println!("{raw}"),
        }
        Ok(())
    }

    /// `challenge` / `detect`: report anti-bot / CAPTCHA markers.
    pub(crate) async fn challenge(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let report = self.page.detect_challenge().await?;
        println!("{}", report.to_json());
        Ok(())
    }

    /// `await-human [s]` / `await` / `human`.
    pub(crate) async fn await_human(
        &mut self,
        rest: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Pause for a person to clear an anti-bot challenge in the browser, then
        // continue. xcelerate detects challenges; it does not solve them. The
        // input gate is lowered first, or the person could not click the
        // challenge either.
        let limit = rest.parse::<u64>().unwrap_or(60);
        crate::cursor::set_gate(&self.page, false).await;
        println!("waiting up to {limit}s for a human to clear the challenge...");
        let started = std::time::Instant::now();
        loop {
            let report = self.page.detect_challenge().await?;
            if !report.detected {
                println!("challenge cleared after {}s", started.elapsed().as_secs());
                break;
            }
            if started.elapsed().as_secs() >= limit {
                println!(
                    "still challenged after {limit}s ({})",
                    report.vendors.join(", ")
                );
                break;
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        // Re-raise the gate now that the human is done.
        crate::cursor::set_gate(&self.page, true).await;
        Ok(())
    }

    /// `guard <path-to-js>`: block popups/ads on this page and every new one.
    pub(crate) async fn guard(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if rest.is_empty() {
            println!("usage: guard <path-to-js>");
            return Ok(());
        }
        match std::fs::read_to_string(rest) {
            Ok(source) => {
                self.page
                    .add_script_to_evaluate_on_new_document(source.clone())
                    .await?;
                // Also run it against the page that is already loaded.
                let _ = self.page.evaluate_json(source).await;
                println!("guard installed from {rest} (this page and every new document)");
            }
            Err(error) => println!("cannot read {rest}: {error}"),
        }
        Ok(())
    }
}
