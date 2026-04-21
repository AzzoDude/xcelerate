//! # xcelerate
//! 
//! `xcelerate` is a high-performance, lightweight Chrome DevTools Protocol (CDP) client.
//! It provides a fluent, chained API for browser automation, designed for speed and reliability.

pub mod error;
pub mod connection;
pub mod browser;
pub mod page;
pub mod element;
pub mod stealth;

pub use error::{XcelerateError, XcelerateResult};
pub use browser::{Browser, BrowserConfig};
pub use page::Page;
pub use element::Element;
pub use connection::{CdpClient, CdpHandler};

uniffi::setup_scaffolding!("xcelerate_core");

/// The core trait for defining CDP commands.
pub use connection::client::CdpCommand;

macro_rules! impl_cdp_command {
    ($params:ty, $returns:ty, $method:expr) => {
        impl CdpCommand for $params {
            type Response = $returns;
            const METHOD: &'static str = $method;
        }
    };
}

impl_cdp_command!(browser_protocol::page::NavigateParams, browser_protocol::page::NavigateReturns, "Page.navigate");
impl_cdp_command!(browser_protocol::page::EnableParams, serde_json::Value, "Page.enable");
impl_cdp_command!(browser_protocol::target::CreateTargetParams, browser_protocol::target::CreateTargetReturns, "Target.createTarget");
impl_cdp_command!(browser_protocol::target::AttachToTargetParams, browser_protocol::target::AttachToTargetReturns, "Target.attachToTarget");
impl_cdp_command!(js_protocol::runtime::EvaluateParams, js_protocol::runtime::EvaluateReturns, "Runtime.evaluate");
impl_cdp_command!(js_protocol::runtime::CallFunctionOnParams, js_protocol::runtime::CallFunctionOnReturns, "Runtime.callFunctionOn");
impl_cdp_command!(browser_protocol::browser::GetVersionParams, browser_protocol::browser::GetVersionReturns, "Browser.getVersion");
impl_cdp_command!(browser_protocol::browser::CloseParams, serde_json::Value, "Browser.close");
impl_cdp_command!(browser_protocol::page::ReloadParams, serde_json::Value, "Page.reload");
impl_cdp_command!(browser_protocol::page::CaptureScreenshotParams, browser_protocol::page::CaptureScreenshotReturns, "Page.captureScreenshot");
impl_cdp_command!(browser_protocol::page::PrintToPDFParams, browser_protocol::page::PrintToPDFReturns, "Page.printToPDF");
impl_cdp_command!(browser_protocol::page::GetNavigationHistoryParams, browser_protocol::page::GetNavigationHistoryReturns, "Page.getNavigationHistory");
impl_cdp_command!(browser_protocol::page::NavigateToHistoryEntryParams, serde_json::Value, "Page.navigateToHistoryEntry");
impl_cdp_command!(browser_protocol::network::EnableParams, serde_json::Value, "Network.enable");
impl_cdp_command!(browser_protocol::target::GetTargetsParams, browser_protocol::target::GetTargetsReturns, "Target.getTargets");
impl_cdp_command!(browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams, browser_protocol::page::AddScriptToEvaluateOnNewDocumentReturns, "Page.addScriptToEvaluateOnNewDocument");
impl_cdp_command!(browser_protocol::page::GetLayoutMetricsParams, browser_protocol::page::GetLayoutMetricsReturns, "Page.getLayoutMetrics");
impl_cdp_command!(browser_protocol::emulation::SetDeviceMetricsOverrideParams, serde_json::Value, "Emulation.setDeviceMetricsOverride");
impl_cdp_command!(browser_protocol::emulation::ClearDeviceMetricsOverrideParams, serde_json::Value, "Emulation.clearDeviceMetricsOverride");
impl_cdp_command!(browser_protocol::input::DispatchKeyEventParams, serde_json::Value, "Input.dispatchKeyEvent");
