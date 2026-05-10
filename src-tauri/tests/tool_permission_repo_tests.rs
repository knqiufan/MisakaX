use rusqlite::Connection;
use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::ToolPermissionRepo;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    conn
}

#[test]
fn test_find_policy_not_found() {
    let conn = create_test_db();
    let result = ToolPermissionRepo::find_policy(&conn, "server1", "read_file").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_upsert_and_find_policy() {
    let conn = create_test_db();

    ToolPermissionRepo::upsert_policy(&conn, "server1", "read_file", "allow").unwrap();
    let result = ToolPermissionRepo::find_policy(&conn, "server1", "read_file").unwrap();
    assert_eq!(result, Some("allow".to_string()));
}

#[test]
fn test_upsert_overwrites_policy() {
    let conn = create_test_db();

    ToolPermissionRepo::upsert_policy(&conn, "server1", "exec_cmd", "allow").unwrap();
    ToolPermissionRepo::upsert_policy(&conn, "server1", "exec_cmd", "deny").unwrap();

    let result = ToolPermissionRepo::find_policy(&conn, "server1", "exec_cmd").unwrap();
    assert_eq!(result, Some("deny".to_string()));
}

#[test]
fn test_list_all() {
    let conn = create_test_db();

    ToolPermissionRepo::upsert_policy(&conn, "s1", "tool_a", "allow").unwrap();
    ToolPermissionRepo::upsert_policy(&conn, "s1", "tool_b", "deny").unwrap();
    ToolPermissionRepo::upsert_policy(&conn, "s2", "tool_c", "ask").unwrap();

    let all = ToolPermissionRepo::list_all(&conn).unwrap();
    assert_eq!(all.len(), 3);

    let policies: Vec<&str> = all.iter().map(|p| p.policy.as_str()).collect();
    assert!(policies.contains(&"allow"));
    assert!(policies.contains(&"deny"));
    assert!(policies.contains(&"ask"));
}

#[test]
fn test_reset_removes_policy() {
    let conn = create_test_db();

    ToolPermissionRepo::upsert_policy(&conn, "server1", "read_file", "allow").unwrap();
    ToolPermissionRepo::reset(&conn, "server1", "read_file").unwrap();

    let result = ToolPermissionRepo::find_policy(&conn, "server1", "read_file").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_reset_nonexistent_is_ok() {
    let conn = create_test_db();
    let result = ToolPermissionRepo::reset(&conn, "ghost", "phantom_tool");
    assert!(result.is_ok());
}

#[test]
fn test_different_servers_same_tool() {
    let conn = create_test_db();

    ToolPermissionRepo::upsert_policy(&conn, "s1", "tool_x", "allow").unwrap();
    ToolPermissionRepo::upsert_policy(&conn, "s2", "tool_x", "deny").unwrap();

    let p1 = ToolPermissionRepo::find_policy(&conn, "s1", "tool_x").unwrap();
    let p2 = ToolPermissionRepo::find_policy(&conn, "s2", "tool_x").unwrap();

    assert_eq!(p1, Some("allow".to_string()));
    assert_eq!(p2, Some("deny".to_string()));
}
