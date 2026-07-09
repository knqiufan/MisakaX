use misaka_x_lib::services::mcp::approval::PolicyDecision;
use misaka_x_lib::services::mcp_http_bridge::{
    bridge_bind_addr, http_policy_decision, map_call_failure_status, validate_call_tool_request,
    CallFailureKind, CallToolRequest,
};

#[test]
fn validate_call_tool_request_requires_fields() {
    let ok = CallToolRequest {
        server_id: "srv".into(),
        tool_name: "tool".into(),
        arguments: serde_json::json!({}),
    };
    assert!(validate_call_tool_request(&ok).is_ok());

    let bad = CallToolRequest {
        server_id: " ".into(),
        tool_name: "tool".into(),
        arguments: serde_json::json!({}),
    };
    assert!(validate_call_tool_request(&bad).is_err());
}

#[test]
fn ask_policy_is_not_silently_allowed() {
    assert_eq!(http_policy_decision(None), PolicyDecision::Ask);
    assert_eq!(http_policy_decision(Some("ask")), PolicyDecision::Ask);
    assert_eq!(http_policy_decision(Some("allow")), PolicyDecision::Allow);
    assert_eq!(http_policy_decision(Some("deny")), PolicyDecision::Deny);
}

#[test]
fn bridge_bind_addr_is_localhost_only() {
    let addr = bridge_bind_addr(9528);
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_eq!(addr.port(), 9528);
}

#[test]
fn call_failure_status_mapping() {
    assert_eq!(map_call_failure_status(CallFailureKind::BadRequest), 400);
    assert_eq!(map_call_failure_status(CallFailureKind::Forbidden), 403);
    assert_eq!(map_call_failure_status(CallFailureKind::Upstream), 502);
}
