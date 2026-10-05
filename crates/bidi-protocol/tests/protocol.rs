//! Round-trip checks for the generated BiDi protocol types.

use bidi_protocol::{BidiCommand, BidiEvent, browsing_context, session};

#[test]
fn serializes_a_command_envelope() {
    let params = browsing_context::NavigateParameters {
        context: "ctx-1".into(),
        url: "https://example.com".into(),
        wait: Some(browsing_context::ReadinessState::Complete),
    };
    let command = browsing_context::Navigate::command(7, &params);
    let json = serde_json::to_value(&command).unwrap();

    assert_eq!(json["id"], 7);
    assert_eq!(json["method"], "browsingContext.navigate");
    assert_eq!(json["params"]["url"], "https://example.com");
    assert_eq!(json["params"]["wait"], "complete");
    assert_eq!(
        browsing_context::Navigate::METHOD,
        "browsingContext.navigate"
    );
}

#[test]
fn deserializes_a_session_new_result() {
    let raw = r#"{
        "id": 1,
        "type": "success",
        "result": {
            "sessionId": "s1",
            "capabilities": {
                "acceptInsecureCerts": false,
                "browserName": "firefox",
                "browserVersion": "128.0",
                "platformName": "windows",
                "setWindowRect": true,
                "userAgent": "Mozilla/5.0"
            }
        }
    }"#;
    let reply: bidi_protocol::SuccessResponse<session::NewResult> =
        serde_json::from_str(raw).unwrap();
    assert_eq!(reply.result.session_id, "s1");
    assert_eq!(reply.result.capabilities.browser_name, "firefox");
}

#[test]
fn carries_command_and_event_metadata() {
    assert_eq!(session::New::METHOD, "session.new");
    assert_eq!(session::Subscribe::METHOD, "session.subscribe");
    assert_eq!(
        browsing_context::Navigate::METHOD,
        "browsingContext.navigate"
    );
    assert_eq!(browsing_context::Load::METHOD, "browsingContext.load");
    assert_eq!(
        browsing_context::ContextCreated::METHOD,
        "browsingContext.contextCreated"
    );
}
