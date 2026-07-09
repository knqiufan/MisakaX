//! MCP 审批服务测试：覆盖 policy allow/deny/ask 三分支判定
//!
//! 审批弹窗（emit + oneshot 等待）依赖真实 `AppHandle`，此处仅对纯判定逻辑
//! `decide_from_policy` 做单元测试；DB 存储由 `tool_permission_repo_tests` 覆盖。

use misaka_x_lib::services::mcp::approval::{decide_from_policy, PolicyDecision};

#[test]
fn allow_policy_maps_to_allow() {
    assert_eq!(decide_from_policy(Some("allow")), PolicyDecision::Allow);
}

#[test]
fn deny_policy_maps_to_deny() {
    assert_eq!(decide_from_policy(Some("deny")), PolicyDecision::Deny);
}

#[test]
fn missing_policy_maps_to_ask() {
    assert_eq!(decide_from_policy(None), PolicyDecision::Ask);
}

#[test]
fn explicit_ask_policy_maps_to_ask() {
    assert_eq!(decide_from_policy(Some("ask")), PolicyDecision::Ask);
}

#[test]
fn unknown_policy_falls_back_to_ask() {
    assert_eq!(decide_from_policy(Some("something-else")), PolicyDecision::Ask);
}
