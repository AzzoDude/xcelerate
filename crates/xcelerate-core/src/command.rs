use serde::Serialize;
use serde::de::DeserializeOwned;

/// A CDP command: the wire method name plus the owned response it decodes into.
///
/// Responses are owned (`'static`) because [`CdpClient`](crate::CdpClient)
/// decodes from an owned [`serde_json::Value`]; parameters may borrow.
pub trait CdpCommand: Serialize {
    const METHOD: &'static str;
    type Response: DeserializeOwned;
}

/// Registers commands by deriving their method + response from the generated
/// protocol crate's own `CdpCommand` derive. This removes all hand-written
/// method strings and response types, so the registry can never drift out of
/// sync with the protocol.
///
/// `lt` marks parameter structs that carry a lifetime, `plain` marks those that
/// do not.
macro_rules! register {
    ($proto:ident; lt $($module:ident::$name:ident),+ $(,)?) => {
        $(
            impl<'a> CdpCommand for $proto::$module::$name<'a> {
                const METHOD: &'static str =
                    <$proto::$module::$name<'static> as $proto::CdpCommand<'static>>::METHOD;
                type Response =
                    <$proto::$module::$name<'static> as $proto::CdpCommand<'static>>::Response;
            }
        )+
    };
    ($proto:ident; plain $($module:ident::$name:ident),+ $(,)?) => {
        $(
            impl CdpCommand for $proto::$module::$name {
                const METHOD: &'static str =
                    <$proto::$module::$name as $proto::CdpCommand<'static>>::METHOD;
                type Response =
                    <$proto::$module::$name as $proto::CdpCommand<'static>>::Response;
            }
        )+
    };
}

register!(browser_protocol; lt
    page::NavigateParams,
    target::CreateTargetParams,
    target::AttachToTargetParams,
    page::ReloadParams,
    page::CaptureScreenshotParams,
    page::PrintToPDFParams,
    target::GetTargetsParams,
    page::AddScriptToEvaluateOnNewDocumentParams,
    emulation::SetDeviceMetricsOverrideParams,
    input::DispatchKeyEventParams,
    input::DispatchMouseEventParams,
    emulation::SetEmulatedMediaParams,
    dom::SetFileInputFilesParams,
    emulation::SetUserAgentOverrideParams,
    network::GetCookiesParams,
    network::LoadNetworkResourceParams,
    io::ReadParams,
    io::CloseParams,
);

register!(browser_protocol; plain
    page::EnableParams,
    browser::GetVersionParams,
    browser::CloseParams,
    page::GetNavigationHistoryParams,
    page::NavigateToHistoryEntryParams,
    network::EnableParams,
    page::GetLayoutMetricsParams,
    emulation::ClearDeviceMetricsOverrideParams,
    page::CloseParams,
    page::BringToFrontParams,
    network::SetExtraHTTPHeadersParams,
    network::SetCacheDisabledParams,
    emulation::SetScriptExecutionDisabledParams,
    network::EmulateNetworkConditionsParams,
    performance::GetMetricsParams,
);

register!(js_protocol; lt
    runtime::EvaluateParams,
    runtime::CallFunctionOnParams,
    runtime::GetPropertiesParams,
);
