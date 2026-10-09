//! Native pointer drag-and-drop (capability #7).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out of
//! the `#[uniffi::export]` block and binding checksums remain stable.

use std::sync::Arc;

use browser_protocol::input::{DispatchMouseEventParams, MouseButton};

use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

/// How many intermediate points a drag moves through. Several moves are needed
/// for HTML5 drag targets to register the drag rather than a single jump.
const DRAG_STEPS: usize = 12;

impl Page {
    /// Drags from the element matching `from` to the element matching `to`.
    ///
    /// `from`/`to` may be any selector the shared resolver understands (CSS,
    /// `xpath=`, `role=`, `text=`, `label=`). The drag is a native pointer drag:
    /// move to the source, press the left button, move in steps, release over the
    /// target.
    pub async fn drag(self: Arc<Self>, from: &str, to: &str) -> XcelerateResult<()> {
        let source = self.clone().resolve_selector(from.to_string()).await?;
        let target = self.clone().resolve_selector(to.to_string()).await?;
        let (from_x, from_y) = source.action_point().await?;
        let (to_x, to_y) = target.action_point().await?;
        self.drag_mouse(from_x, from_y, to_x, to_y).await
    }

    /// Drags the element matching `selector` onto raw viewport coordinates.
    pub async fn drag_to(
        self: Arc<Self>,
        selector: &str,
        to_x: f64,
        to_y: f64,
    ) -> XcelerateResult<()> {
        let source = self.clone().resolve_selector(selector.to_string()).await?;
        let (from_x, from_y) = source.action_point().await?;
        self.drag_mouse(from_x, from_y, to_x, to_y).await
    }

    /// Performs a native pointer drag between two viewport points.
    pub async fn drag_mouse(
        self: Arc<Self>,
        from_x: f64,
        from_y: f64,
        to_x: f64,
        to_y: f64,
    ) -> XcelerateResult<()> {
        self.dispatch_mouse("mouseMoved", from_x, from_y, None, 0)
            .await?;
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        self.dispatch_mouse("mousePressed", from_x, from_y, Some(MouseButton::Left), 1)
            .await?;
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;

        for step in 1..=DRAG_STEPS {
            let progress = step as f64 / DRAG_STEPS as f64;
            let x = from_x + (to_x - from_x) * progress;
            let y = from_y + (to_y - from_y) * progress;
            self.dispatch_mouse("mouseMoved", x, y, None, 1).await?;
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
        }

        self.dispatch_mouse("mouseReleased", to_x, to_y, Some(MouseButton::Left), 0)
            .await?;
        // Keep the page's tracked cursor in sync for later actions.
        self.set_mouse_position(to_x, to_y);
        Ok(())
    }

    /// Dispatches a single mouse event with an explicit button bitmask.
    async fn dispatch_mouse(
        &self,
        type_: &str,
        x: f64,
        y: f64,
        button: Option<MouseButton>,
        buttons: i64,
    ) -> XcelerateResult<()> {
        let params = DispatchMouseEventParams {
            type_: type_.into(),
            x,
            y,
            button,
            buttons: Some(buttons),
            click_count: Some(1),
            ..Default::default()
        };
        self.client
            .execute_with_session(Some(&self.session_id), params)
            .await
            .map_err(XcelerateError::from)?;
        Ok(())
    }
}
