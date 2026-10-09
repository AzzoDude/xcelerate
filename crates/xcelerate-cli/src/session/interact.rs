//! Verbs that drive the page: clicks, typing, keys and the mouse.

use std::sync::Arc;
use std::time::Duration;

use xcelerate_interpreter::interact::{control_by_text, looks_like_a_selector};

use super::input::{
    parse_coordinates, report_no_text_match, split_mouse_action, split_selector_text,
};
use super::state::Session;

impl Session {
    /// `click <index|selector|text>`.
    pub(crate) async fn click(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if rest.is_empty() {
            println!("usage: click <index|selector|text>");
            return Ok(());
        }
        if let Ok(index) = rest.parse::<u32>() {
            crate::cursor::set_driving(&self.page, true).await;
            // The index map only exists after a snapshot; build it on
            // first use so `click 3` works even if `snapshot` was never run.
            if Arc::clone(&self.page).click_index(index).await.is_err() {
                let _ = self.page.snapshot_json().await?;
                Arc::clone(&self.page).click_index(index).await?;
            }
            tokio::time::sleep(Duration::from_millis(600)).await;
            println!(
                "clicked [{index}] -> {}",
                self.page.url().await.unwrap_or_default()
            );
        } else if looks_like_a_selector(rest) {
            let element = Arc::clone(&self.page)
                .wait_for_selector(rest.to_string())
                .await?;
            crate::cursor::set_driving(&self.page, true).await;
            element.click_mouse().await?;
            tokio::time::sleep(Duration::from_millis(600)).await;
            println!(
                "clicked {rest} -> {}",
                self.page.url().await.unwrap_or_default()
            );
        } else {
            // Neither a snapshot index nor a selector: match visible text.
            match control_by_text(&self.page, rest).await? {
                Some(element) => {
                    crate::cursor::set_driving(&self.page, true).await;
                    element.click_mouse().await?;
                    tokio::time::sleep(Duration::from_millis(600)).await;
                    println!(
                        "clicked {rest:?} -> {}",
                        self.page.url().await.unwrap_or_default()
                    );
                }
                None => {
                    report_no_text_match(rest);
                    self.step_no_op = true;
                }
            }
        }
        Ok(())
    }

    /// `dialog <dismiss|accept>`: how JS dialogs are answered automatically.
    ///
    /// Every page dismisses dialogs by default, so a stray `alert`/`confirm` can
    /// never wedge a run; `accept` accepts them instead (for `beforeunload`).
    pub(crate) async fn dialog(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        let policy = match rest.trim().to_ascii_lowercase().as_str() {
            "" | "dismiss" => xcelerate::page::DialogPolicy::Dismiss,
            "accept" => xcelerate::page::DialogPolicy::Accept,
            other => {
                println!("usage: dialog <dismiss|accept>   (got {other:?})");
                return Ok(());
            }
        };
        self.page.set_dialog_policy(policy).await?;
        println!(
            "dialogs: {}",
            if rest.trim().is_empty() {
                "dismiss"
            } else {
                rest.trim()
            }
        );
        Ok(())
    }

    /// `drag <from> <to>`: a native pointer drag between two elements.
    pub(crate) async fn drag(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut parts = rest.split_whitespace();
        let (Some(from), Some(to)) = (parts.next(), parts.next()) else {
            println!("usage: drag <from-selector|text> <to-selector|text>");
            return Ok(());
        };
        Arc::clone(&self.page).drag(from, to).await?;
        println!("dragged {from} -> {to}");
        Ok(())
    }

    /// `tap <selector|text>`: a DOM click that never moves the mouse.
    pub(crate) async fn tap(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if rest.is_empty() {
            println!("usage: tap <selector|text>   (DOM click, no mouse movement)");
        } else if looks_like_a_selector(rest) {
            let element = Arc::clone(&self.page)
                .wait_for_selector(rest.to_string())
                .await?;
            // Lower the gate first: it blocks click events, and a DOM
            // click is still a click event.
            crate::cursor::set_driving(&self.page, true).await;
            element.click().await?;
            println!("tapped {rest}");
        } else {
            match control_by_text(&self.page, rest).await? {
                Some(element) => {
                    crate::cursor::set_driving(&self.page, true).await;
                    element.click().await?;
                    println!("tapped {rest:?}");
                }
                None => {
                    report_no_text_match(rest);
                    self.step_no_op = true;
                }
            }
        }
        Ok(())
    }

    /// `fill <selector|index> <text>`: focus and type slowly.
    pub(crate) async fn fill(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        match split_selector_text(tokens) {
            Some((selector, text)) => {
                crate::cursor::set_driving(&self.page, true).await;
                // A bare integer is a snapshot index: many framework-
                // rendered fields (e.g. Facebook signup) expose no stable
                // selector. Focus it by index, then type.
                let element = if let Ok(index) = selector.parse::<u32>() {
                    if Arc::clone(&self.page).click_index(index).await.is_err() {
                        let _ = self.page.snapshot_json().await;
                        Arc::clone(&self.page).click_index(index).await?;
                    }
                    Arc::clone(&self.page)
                        .evaluate_handle("document.activeElement".to_string())
                        .await?
                } else {
                    Arc::clone(&self.page)
                        .wait_for_selector(selector.clone())
                        .await?
                };
                let count = text.chars().count();
                // A disabled or read-only field silently drops typed text; report
                // it as a miss instead of claiming `typed N chars`.
                if element
                    .evaluate_bool(
                        "function(){ return this.disabled === true || this.readOnly === true; }"
                            .to_string(),
                    )
                    .await
                    .unwrap_or(false)
                {
                    println!("{selector} is disabled or read-only; nothing typed");
                    self.step_no_op = true;
                    return Ok(());
                }
                element.type_text(text).await?;
                println!("typed {count} chars into {selector}");
                Ok(())
            }
            None => {
                println!("usage: fill <selector|index> <text>");
                Ok(())
            }
        }
    }

    /// `select <selector> <value>`: choose an option in a native `<select>`.
    pub(crate) async fn select(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        match split_selector_text(tokens) {
            Some((selector, value)) => {
                // Native `<select>` only (matches option by value or label).
                // For a custom listbox, click the control to open it, then
                // `click "<option>"`.
                let values = serde_json::json!([value]).to_string();
                Arc::clone(&self.page)
                    .select_option(selector.clone(), values)
                    .await?;
                println!("selected {value:?} in {selector}");
                Ok(())
            }
            None => {
                println!("usage: select <selector> <value>   (native <select>)");
                Ok(())
            }
        }
    }

    /// `type <text>`: type into the focused element.
    pub(crate) async fn type_text(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if rest.is_empty() {
            println!("usage: type <text>   (types into the focused element)");
        } else {
            let element = Arc::clone(&self.page)
                .evaluate_handle("document.activeElement".to_string())
                .await?;
            crate::cursor::set_driving(&self.page, true).await;
            let count = rest.chars().count();
            element.type_text(rest.to_string()).await?;
            println!("typed {count} chars into the focused element");
        }
        Ok(())
    }

    /// `press <key>` / `submit` / `send`: a key on the focused element.
    pub(crate) async fn press(
        &mut self,
        verb: &str,
        rest: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // `submit` is the productive path: finish typing, then submit
        // from the field you are already in instead of hunting for a
        // Send button. It is Enter on the focused element.
        let key = if verb == "press" {
            rest.to_string()
        } else {
            "Enter".to_string()
        };
        if key.is_empty() {
            println!("usage: press <key>   (sends to the focused element)");
            return Ok(());
        }
        let element = Arc::clone(&self.page)
            .evaluate_handle("document.activeElement".to_string())
            .await?;
        crate::cursor::set_driving(&self.page, true).await;
        element.press(key.clone()).await?;
        println!("pressed {key}");
        Ok(())
    }

    /// `hover <selector>`: move the mouse over an element.
    pub(crate) async fn hover(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if rest.is_empty() {
            println!("usage: hover <selector>");
        } else {
            let element = Arc::clone(&self.page)
                .wait_for_selector(rest.to_string())
                .await?;
            crate::cursor::set_driving(&self.page, true).await;
            element.hover_mouse().await?;
            println!("hovered {rest}");
        }
        Ok(())
    }

    /// `mouse [click] <index|selector|text> | mouse <x> <y>`.
    pub(crate) async fn mouse(
        &mut self,
        tokens: &[String],
        rest: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Move the real cursor; `mouse click <target>` moves the mouse to
        // the target and then clicks it. The target is a snapshot index,
        // CSS selector, visible text, or raw `x y` coordinates.
        let (click, target) = split_mouse_action(tokens, rest);
        if target.is_empty() {
            println!("usage: mouse [click] <index|selector|text> | mouse <x> <y>");
        } else {
            crate::cursor::set_driving(&self.page, true).await;
            if let Some((x, y)) = parse_coordinates(&target) {
                if click {
                    Arc::clone(&self.page).click_mouse(x, y).await?;
                    println!("clicked at ({x}, {y})");
                } else {
                    Arc::clone(&self.page).move_mouse(x, y).await?;
                    println!("moved to ({x}, {y})");
                }
            } else if let Ok(index) = target.parse::<u32>() {
                if click {
                    if Arc::clone(&self.page).click_index(index).await.is_err() {
                        let _ = self.page.snapshot_json().await?;
                        Arc::clone(&self.page).click_index(index).await?;
                    }
                    println!("clicked [{index}]");
                } else {
                    // The index map only exists after a snapshot; build
                    // it on first use so `mouse 3` works without one.
                    if Arc::clone(&self.page).move_to_index(index).await.is_err() {
                        let _ = self.page.snapshot_json().await?;
                        Arc::clone(&self.page).move_to_index(index).await?;
                    }
                    println!("moved to [{index}]");
                }
            } else if looks_like_a_selector(&target) {
                let element = Arc::clone(&self.page)
                    .wait_for_selector(target.clone())
                    .await?;
                if click {
                    element.click_mouse().await?;
                    println!("clicked {target}");
                } else {
                    element.hover_mouse().await?;
                    println!("moved to {target}");
                }
            } else {
                match control_by_text(&self.page, &target).await? {
                    Some(element) => {
                        if click {
                            element.click_mouse().await?;
                            println!("clicked {target:?}");
                        } else {
                            element.hover_mouse().await?;
                            println!("moved to {target:?}");
                        }
                    }
                    None => {
                        report_no_text_match(&target);
                        self.step_no_op = true;
                    }
                }
            }
        }
        Ok(())
    }

    /// `click-xy <x> <y>` / `click-at` / `xy`: raw coordinate click.
    pub(crate) async fn click_xy(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Raw coordinate click: for canvases, maps and embedded
        // (cross-origin) widgets that DOM selectors cannot reach.
        let mut parts = rest.split_whitespace();
        let x = parts.next().and_then(|v| v.parse::<f64>().ok());
        let y = parts.next().and_then(|v| v.parse::<f64>().ok());
        match (x, y) {
            (Some(x), Some(y)) => {
                crate::cursor::set_driving(&self.page, true).await;
                Arc::clone(&self.page).click_mouse(x, y).await?;
                println!("clicked at ({x}, {y})");
            }
            _ => println!("usage: click-xy <x> <y>"),
        }
        Ok(())
    }
}
