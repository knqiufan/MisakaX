//! MCP 对话工具循环——编排层集成测试
//!
//! 不拉起真实 Rig / 网络 / AppHandle，只覆盖工具循环依赖的可测边界：
//! 1. 解析 → strip → 写库 → 读回（工具轮 assistant content 的持久化路径）
//! 2. 审批 policy 落库 → 判定（allow/deny/ask 的存储编排）
//! 3. `ToolCallRecord` 序列化字段与前端 `ToolCall` 契约对齐
//! 4. `stream:tool_call` / `stream:tool_result` payload 形状对齐前端事件

use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::{MessageRepo, ToolPermissionRepo};
use misaka_x_lib::services::llm::{StreamToolCallPayload, StreamToolResultPayload};
use misaka_x_lib::services::mcp::approval::{decide_from_policy, PolicyDecision};
use misaka_x_lib::services::mcp::tool_loop::{parse_tool_call_from_content, strip_tool_call_json};
use misaka_x_lib::services::ToolCallRecord;
use rusqlite::Connection;
use serde_json::{json, Value};

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    conn.execute("INSERT INTO sessions (id, title) VALUES ('s1', 'Test')", [])
        .unwrap();
    conn
}

fn sample_record() -> ToolCallRecord {
    ToolCallRecord {
        id: "tc1".to_string(),
        server_id: "srv".to_string(),
        server_name: "Filesystem".to_string(),
        tool_name: "read_file".to_string(),
        arguments: json!({ "path": "package.json" }),
        result: Some(json!({ "content": "ok" })),
        status: "complete".to_string(),
        error: None,
        started_at: Some(1000),
        completed_at: Some(1200),
    }
}

// ─── 1. 解析 → strip → 写库 → 读回 ────────────────────────────────────

#[test]
fn tool_round_content_parses_and_persists_record() {
    let conn = setup_db();
    MessageRepo::insert_assistant_placeholder(&conn, "m1", "s1", "gpt-4o").unwrap();

    // 模拟模型在工具轮输出的 fenced 工具调用
    let content = "Let me read it.\n```json\n{\"name\":\"read_file\",\"arguments\":{\"path\":\"package.json\"}}\n```";

    let call = parse_tool_call_from_content(content).expect("tool call parsed");
    assert_eq!(call.name, "read_file");
    assert_eq!(call.arguments["path"], "package.json");

    // 可见文本应剥离工具 JSON，仅保留正文
    assert_eq!(strip_tool_call_json(content), "Let me read it.");

    // 用解析出的调用构造记录并写入 messages.tool_calls
    let record = ToolCallRecord {
        tool_name: call.name.clone(),
        arguments: call.arguments.clone(),
        ..sample_record()
    };
    let serialized = serde_json::to_string(&[record]).unwrap();
    MessageRepo::update_tool_calls(&conn, "m1", &serialized).unwrap();

    let messages = MessageRepo::find_recent(&conn, "s1", 10).unwrap();
    let raw = messages[0]
        .tool_calls
        .as_deref()
        .expect("tool_calls persisted");
    let parsed: Vec<Value> = serde_json::from_str(raw).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["tool_name"], "read_file");
    assert_eq!(parsed[0]["status"], "complete");
    assert_eq!(parsed[0]["arguments"]["path"], "package.json");
}

#[test]
fn final_answer_round_has_no_tool_call() {
    // 最终纯文本回答轮：解析为 None，可见文本原样保留
    let content = "The file declares version 0.1.0.";
    assert!(parse_tool_call_from_content(content).is_none());
    assert_eq!(strip_tool_call_json(content), content);
}

// ─── 2. 审批 policy 落库 → 判定 ───────────────────────────────────────

#[test]
fn approval_allow_policy_persists_and_decides_allow() {
    let conn = setup_db();
    ToolPermissionRepo::upsert_policy(&conn, "srv", "read_file", "allow").unwrap();

    let policy = ToolPermissionRepo::find_policy(&conn, "srv", "read_file").unwrap();
    assert_eq!(policy.as_deref(), Some("allow"));
    assert_eq!(decide_from_policy(policy.as_deref()), PolicyDecision::Allow);
}

#[test]
fn approval_deny_policy_persists_and_decides_deny() {
    let conn = setup_db();
    ToolPermissionRepo::upsert_policy(&conn, "srv", "delete_file", "deny").unwrap();

    let policy = ToolPermissionRepo::find_policy(&conn, "srv", "delete_file").unwrap();
    assert_eq!(policy.as_deref(), Some("deny"));
    assert_eq!(decide_from_policy(policy.as_deref()), PolicyDecision::Deny);
}

#[test]
fn approval_absent_policy_decides_ask() {
    let conn = setup_db();
    // 未落库的工具调用应回落到 Ask（弹窗）
    let policy = ToolPermissionRepo::find_policy(&conn, "srv", "never_seen").unwrap();
    assert!(policy.is_none());
    assert_eq!(decide_from_policy(policy.as_deref()), PolicyDecision::Ask);
}

#[test]
fn approval_policy_upsert_overwrites_previous_decision() {
    let conn = setup_db();
    ToolPermissionRepo::upsert_policy(&conn, "srv", "read_file", "deny").unwrap();
    ToolPermissionRepo::upsert_policy(&conn, "srv", "read_file", "allow").unwrap();

    let policy = ToolPermissionRepo::find_policy(&conn, "srv", "read_file").unwrap();
    assert_eq!(decide_from_policy(policy.as_deref()), PolicyDecision::Allow);
}

// ─── 3. ToolCallRecord 序列化契约 ─────────────────────────────────────

#[test]
fn tool_call_record_serializes_frontend_contract() {
    let value = serde_json::to_value(sample_record()).unwrap();
    assert_eq!(
        value,
        json!({
            "id": "tc1",
            "server_id": "srv",
            "server_name": "Filesystem",
            "tool_name": "read_file",
            "arguments": { "path": "package.json" },
            "result": { "content": "ok" },
            "status": "complete",
            "error": null,
            "started_at": 1000,
            "completed_at": 1200
        })
    );
}

#[test]
fn tool_call_record_error_variant_keeps_null_result() {
    let record = ToolCallRecord {
        result: None,
        status: "error".to_string(),
        error: Some("Tool call denied".to_string()),
        ..sample_record()
    };
    let value = serde_json::to_value(record).unwrap();
    assert_eq!(value["status"], "error");
    assert_eq!(value["result"], Value::Null);
    assert_eq!(value["error"], "Tool call denied");
}

// ─── 4. 流式事件 payload 形状 ─────────────────────────────────────────

#[test]
fn stream_tool_call_payload_matches_frontend_event() {
    let payload = StreamToolCallPayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        tool_call_id: "tc1".to_string(),
        server_id: "srv".to_string(),
        server_name: "Filesystem".to_string(),
        tool_name: "read_file".to_string(),
        arguments: json!({ "path": "package.json" }),
        status: "running".to_string(),
    };
    let value = serde_json::to_value(&payload).unwrap();
    assert_eq!(value["session_id"], "s1");
    assert_eq!(value["message_id"], "m1");
    assert_eq!(value["tool_call_id"], "tc1");
    assert_eq!(value["server_name"], "Filesystem");
    assert_eq!(value["tool_name"], "read_file");
    assert_eq!(value["arguments"]["path"], "package.json");
    assert_eq!(value["status"], "running");
}

#[test]
fn stream_tool_result_payload_matches_frontend_event() {
    let payload = StreamToolResultPayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        tool_call_id: "tc1".to_string(),
        result: Some(json!({ "content": "ok" })),
        error: None,
        status: "complete".to_string(),
    };
    let value = serde_json::to_value(&payload).unwrap();
    assert_eq!(value["tool_call_id"], "tc1");
    assert_eq!(value["result"]["content"], "ok");
    assert_eq!(value["error"], Value::Null);
    assert_eq!(value["status"], "complete");
}
