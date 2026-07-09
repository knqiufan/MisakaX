//! MCP 工具调用解析器测试（纯函数）
//!
//! 覆盖 `parse_tool_call_from_content` 的多种输出形态，以及 `strip_tool_call_json`
//! 去除工具 JSON 后的可见文本。

use misaka_x_lib::services::mcp::tool_loop::{parse_tool_call_from_content, strip_tool_call_json};

#[test]
fn parse_fenced_json_object() {
    let content =
        "```json\n{\"name\": \"read_file\", \"arguments\": {\"path\": \"package.json\"}}\n```";
    let call = parse_tool_call_from_content(content).expect("should parse fenced tool call");
    assert_eq!(call.name, "read_file");
    assert_eq!(call.arguments["path"], "package.json");
}

#[test]
fn parse_bare_json_object() {
    let content = "{\"name\":\"list_dir\",\"arguments\":{\"path\":\".\"}}";
    let call = parse_tool_call_from_content(content).expect("should parse bare tool call");
    assert_eq!(call.name, "list_dir");
    assert_eq!(call.arguments["path"], ".");
}

#[test]
fn parse_supports_tool_name_and_args_aliases() {
    let content = "{\"tool_name\":\"grep\",\"args\":{\"pattern\":\"fn main\"}}";
    let call = parse_tool_call_from_content(content).expect("should accept aliases");
    assert_eq!(call.name, "grep");
    assert_eq!(call.arguments["pattern"], "fn main");
}

#[test]
fn parse_defaults_missing_arguments_to_empty_object() {
    let content = "{\"name\":\"now\"}";
    let call = parse_tool_call_from_content(content).expect("missing args ok");
    assert_eq!(call.name, "now");
    assert!(call.arguments.is_object());
    assert_eq!(call.arguments.as_object().unwrap().len(), 0);
}

#[test]
fn parse_plain_text_returns_none() {
    assert!(parse_tool_call_from_content("Here is the answer, no tools needed.").is_none());
}

#[test]
fn parse_ignores_non_tool_fenced_code() {
    let content = "Here is code:\n```python\nprint('hi')\n```";
    assert!(parse_tool_call_from_content(content).is_none());
}

#[test]
fn parse_prose_wrapped_fenced_tool_call() {
    let content = "Sure, let me check.\n```json\n{\"name\":\"read_file\",\"arguments\":{}}\n```";
    let call = parse_tool_call_from_content(content).expect("should find fenced call amid prose");
    assert_eq!(call.name, "read_file");
}

#[test]
fn strip_fenced_tool_call_yields_empty() {
    let content = "```json\n{\"name\":\"read_file\",\"arguments\":{\"path\":\"a\"}}\n```";
    assert_eq!(strip_tool_call_json(content), "");
}

#[test]
fn strip_bare_tool_call_yields_empty() {
    let content = "{\"name\":\"read_file\",\"arguments\":{}}";
    assert_eq!(strip_tool_call_json(content), "");
}

#[test]
fn strip_keeps_plain_answer() {
    let content = "The file has 10 lines.";
    assert_eq!(strip_tool_call_json(content), "The file has 10 lines.");
}

#[test]
fn strip_keeps_non_tool_code_block() {
    let content = "Example:\n```python\nprint('hi')\n```";
    assert_eq!(strip_tool_call_json(content), content.trim());
}

#[test]
fn strip_removes_fenced_call_but_keeps_prose() {
    let content = "Let me look.\n```json\n{\"name\":\"read_file\",\"arguments\":{}}\n```";
    assert_eq!(strip_tool_call_json(content), "Let me look.");
}
