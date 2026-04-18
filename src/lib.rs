pub mod error;
pub mod connection;
pub mod browser;
pub mod page;
pub mod element;

pub use error::{XcelerateError, XcelerateResult};
pub use browser::{Browser, BrowserConfig};
pub use page::Page;
pub use element::Element;
pub use connection::{CdpClient, CdpHandler};

// Macro or trait helper for commands
pub use connection::client::CdpCommand;

// Boilerplate trait implementations (usually generated)
impl CdpCommand for browser_protocol::page::NavigateParams {
    type Response = browser_protocol::page::NavigateReturns;
    const METHOD: &'static str = "Page.navigate";
}
impl CdpCommand for browser_protocol::page::EnableParams {
    type Response = ();
    const METHOD: &'static str = "Page.enable";
}
impl CdpCommand for browser_protocol::target::CreateTargetParams {
    type Response = browser_protocol::target::CreateTargetReturns;
    const METHOD: &'static str = "Target.createTarget";
}
impl CdpCommand for browser_protocol::target::AttachToTargetParams {
    type Response = browser_protocol::target::AttachToTargetReturns;
    const METHOD: &'static str = "Target.attachToTarget";
}
impl CdpCommand for js_protocol::runtime::EvaluateParams {
    type Response = js_protocol::runtime::EvaluateReturns;
    const METHOD: &'static str = "Runtime.evaluate";
}
impl CdpCommand for js_protocol::runtime::CallFunctionOnParams {
    type Response = js_protocol::runtime::CallFunctionOnReturns;
    const METHOD: &'static str = "Runtime.callFunctionOn";
}
impl CdpCommand for browser_protocol::browser::GetVersionParams {
    type Response = browser_protocol::browser::GetVersionReturns;
    const METHOD: &'static str = "Browser.getVersion";
}
impl CdpCommand for browser_protocol::browser::CloseParams {
    type Response = ();
    const METHOD: &'static str = "Browser.close";
}
impl CdpCommand for browser_protocol::page::ReloadParams {
    type Response = ();
    const METHOD: &'static str = "Page.reload";
}
impl CdpCommand for browser_protocol::page::CaptureScreenshotParams {
    type Response = browser_protocol::page::CaptureScreenshotReturns;
    const METHOD: &'static str = "Page.captureScreenshot";
}
impl CdpCommand for browser_protocol::page::PrintToPDFParams {
    type Response = browser_protocol::page::PrintToPDFReturns;
    const METHOD: &'static str = "Page.printToPDF";
}
impl CdpCommand for browser_protocol::page::GetNavigationHistoryParams {
    type Response = browser_protocol::page::GetNavigationHistoryReturns;
    const METHOD: &'static str = "Page.getNavigationHistory";
}
impl CdpCommand for browser_protocol::page::NavigateToHistoryEntryParams {
    type Response = ();
    const METHOD: &'static str = "Page.navigateToHistoryEntry";
}
impl CdpCommand for browser_protocol::network::EnableParams {
    type Response = ();
    const METHOD: &'static str = "Network.enable";
}
impl CdpCommand for browser_protocol::target::GetTargetsParams {
    type Response = browser_protocol::target::GetTargetsReturns;
    const METHOD: &'static str = "Target.getTargets";
}
impl CdpCommand for browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams {
    type Response = browser_protocol::page::AddScriptToEvaluateOnNewDocumentReturns;
    const METHOD: &'static str = "Page.addScriptToEvaluateOnNewDocument";
}
