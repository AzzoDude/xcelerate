//! Media and file verbs: screenshots, media listing, downloads and uploads.

use std::sync::Arc;

use xcelerate_interpreter::runtime::resolve_in_root;

use super::help::print_media;
use super::input::split_selector_text;
use super::state::Session;

impl Session {
    /// `shot [path]` / `screenshot`; `full` selects the full-page variant.
    pub(crate) async fn shot(
        &mut self,
        rest: &str,
        full: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let raw = if rest.is_empty() {
            "screenshot.png".to_string()
        } else {
            rest.to_string()
        };
        let path = resolve_in_root(&self.root, &raw)?;
        let png = if full {
            self.page.screenshot_full().await?
        } else {
            self.page.screenshot().await?
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &png)?;
        println!("wrote {} ({} bytes)", path.display(), png.len());
        Ok(())
    }

    /// `media`: list the page's images/video/audio, one line each.
    pub(crate) async fn media(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Enumerate the media the page references, rendered like `snapshot`.
        print_media(&self.page.media_json().await?);
        Ok(())
    }

    /// `download <url> <path>`: fetch a file or HLS stream through the browser.
    pub(crate) async fn download(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // `download <url> <path>`: fetch through the browser. A direct
        // file is streamed; an HLS (`.m3u8`) stream is assembled from
        // its segments.
        match split_selector_text(tokens) {
            Some((url, path)) => {
                let target = resolve_in_root(&self.root, &path)?;
                let summary = self
                    .page
                    .grab(url.clone(), target.to_string_lossy().into_owned())
                    .await?;
                println!("{summary}");
            }
            None => println!("usage: download <url> <path>"),
        }
        Ok(())
    }

    /// `upload <selector> <path>` / `set-input-files`.
    pub(crate) async fn upload(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // `<input type="file">` cannot be set from page JS; this uses
        // CDP `DOM.setFileInputFiles` under the hood. Tokens (not
        // `rest`) so a selector containing spaces stays intact.
        match split_selector_text(tokens) {
            Some((selector, path)) => {
                let source = resolve_in_root(&self.root, &path)?;
                let files = serde_json::json!([source.to_string_lossy()]).to_string();
                Arc::clone(&self.page)
                    .set_input_files(selector.clone(), files)
                    .await?;
                println!("set {selector} <- {}", source.display());
            }
            None => println!("usage: upload <selector> <path>"),
        }
        Ok(())
    }
}
