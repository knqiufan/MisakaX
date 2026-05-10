use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::{SessionRepo, WorkspaceRepo};
use rusqlite::Connection;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    conn
}

// ─── create ─────────────────────────────────────────────────────────

#[test]
fn create_session_without_working_directory() {
    let conn = create_test_db();

    let session = SessionRepo::create(&conn, "sess-1", Some("Test Chat"), None, None).unwrap();

    assert_eq!(session.id, "sess-1");
    assert_eq!(session.title, Some("Test Chat".to_string()));
    assert_eq!(session.working_directory, None);
    assert_eq!(session.project_name, None);
    assert_eq!(session.status, "active");
    assert_eq!(session.mode, "agent");
}

#[test]
fn create_session_with_working_directory() {
    let conn = create_test_db();

    let session = SessionRepo::create(
        &conn,
        "sess-2",
        None,
        Some("config1:gpt-4"),
        Some("/home/user/my-project"),
    )
    .unwrap();

    assert_eq!(session.id, "sess-2");
    assert_eq!(session.working_directory, Some("/home/user/my-project".to_string()));
    assert_eq!(session.project_name, Some("my-project".to_string()));
    assert_eq!(session.model, Some("config1:gpt-4".to_string()));
}

#[test]
fn create_session_extracts_project_name_from_path() {
    let conn = create_test_db();

    let session = SessionRepo::create(
        &conn,
        "sess-3",
        None,
        None,
        Some("D:\\code\\Misaka-Tauri"),
    )
    .unwrap();

    assert_eq!(session.project_name, Some("Misaka-Tauri".to_string()));
}

#[test]
fn create_session_persists_to_sqlite() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-persist", Some("Persisted"), None, Some("/tmp/project"))
        .unwrap();

    let loaded = SessionRepo::find_by_id(&conn, "sess-persist").unwrap();
    assert_eq!(loaded.title, Some("Persisted".to_string()));
    assert_eq!(loaded.working_directory, Some("/tmp/project".to_string()));
    assert_eq!(loaded.project_name, Some("project".to_string()));
}

// ─── update_working_directory ────────────────────────────────────────

#[test]
fn update_working_directory_sets_new_dir() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-upd", None, None, None).unwrap();

    SessionRepo::update_working_directory(&conn, "sess-upd", Some("/new/path/project-x"))
        .unwrap();

    let session = SessionRepo::find_by_id(&conn, "sess-upd").unwrap();
    assert_eq!(session.working_directory, Some("/new/path/project-x".to_string()));
    assert_eq!(session.project_name, Some("project-x".to_string()));
}

#[test]
fn update_working_directory_can_clear() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-clr", None, None, Some("/initial/dir")).unwrap();

    SessionRepo::update_working_directory(&conn, "sess-clr", None).unwrap();

    let session = SessionRepo::find_by_id(&conn, "sess-clr").unwrap();
    assert_eq!(session.working_directory, None);
    assert_eq!(session.project_name, None);
}

// ─── 工作目录绑定与 recent_directories 集成 ─────────────────────────

#[test]
fn workspace_binding_records_usage_in_recent_directories() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-rec", None, None, Some("/projects/awesome")).unwrap();

    let id = uuid::Uuid::new_v4().to_string();
    WorkspaceRepo::record_usage(&conn, &id, "/projects/awesome", Some("awesome")).unwrap();

    let recent = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert!(!recent.is_empty());
    assert!(recent.iter().any(|d| d.path == "/projects/awesome"));
}

// ─── update_stats ───────────────────────────────────────────────────

#[test]
fn update_stats_increments_token_counts() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-stats", None, None, None).unwrap();

    SessionRepo::update_stats(&conn, "sess-stats", Some(100), Some(50)).unwrap();
    SessionRepo::update_stats(&conn, "sess-stats", Some(200), Some(100)).unwrap();

    let total_input: i64 = conn
        .query_row(
            "SELECT total_input_tokens FROM sessions WHERE id = 'sess-stats'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let total_output: i64 = conn
        .query_row(
            "SELECT total_output_tokens FROM sessions WHERE id = 'sess-stats'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(total_input, 300);
    assert_eq!(total_output, 150);
}

#[test]
fn update_stats_without_tokens_updates_timestamp() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-ts", None, None, None).unwrap();

    SessionRepo::update_stats(&conn, "sess-ts", None, None).unwrap();

    let last: Option<String> = conn
        .query_row(
            "SELECT last_message_at FROM sessions WHERE id = 'sess-ts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(last.is_some());
}

// ─── list ─────────────────────────────────────────────────────────────

#[test]
fn list_sessions_returns_active_by_default() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "s1", Some("First"), None, None).unwrap();
    SessionRepo::create(&conn, "s2", Some("Second"), None, None).unwrap();

    let list = SessionRepo::list(&conn, None).unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn list_sessions_filters_by_status() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "s1", Some("Active"), None, None).unwrap();
    SessionRepo::create(&conn, "s2", Some("Archived"), None, None).unwrap();
    SessionRepo::update(&conn, "s2", None, None, None, Some("archived")).unwrap();

    let active = SessionRepo::list(&conn, Some("active")).unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, "s1");

    let archived = SessionRepo::list(&conn, Some("archived")).unwrap();
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].id, "s2");
}

#[test]
fn list_sessions_pinned_first() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "s1", Some("Normal"), None, None).unwrap();
    SessionRepo::create(&conn, "s2", Some("Pinned"), None, None).unwrap();
    SessionRepo::update(&conn, "s2", None, None, Some(true), None).unwrap();

    let list = SessionRepo::list(&conn, None).unwrap();
    assert_eq!(list[0].id, "s2");
    assert!(list[0].pinned);
}

// ─── update ───────────────────────────────────────────────────────────

#[test]
fn update_session_title() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "s1", Some("Old Title"), None, None).unwrap();

    SessionRepo::update(&conn, "s1", Some("New Title"), None, None, None).unwrap();

    let session = SessionRepo::find_by_id(&conn, "s1").unwrap();
    assert_eq!(session.title, Some("New Title".to_string()));
}

#[test]
fn update_session_pin_and_model() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "s1", None, None, None).unwrap();

    SessionRepo::update(&conn, "s1", None, Some("config1:gpt-4"), Some(true), None).unwrap();

    let session = SessionRepo::find_by_id(&conn, "s1").unwrap();
    assert!(session.pinned);
    assert_eq!(session.model, Some("config1:gpt-4".to_string()));
}

// ─── delete ──────────────────────────────────────────────────────────

#[test]
fn delete_session_removes_session_and_messages() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "s1", Some("To Delete"), None, None).unwrap();

    conn.execute(
        "INSERT INTO messages (id, session_id, role, content) VALUES ('m1', 's1', 'user', 'Hello')",
        [],
    )
    .unwrap();

    SessionRepo::delete(&conn, "s1").unwrap();

    let result = SessionRepo::find_by_id(&conn, "s1");
    assert!(result.is_err());

    let msg_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE session_id = 's1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(msg_count, 0);
}

// ─── search ──────────────────────────────────────────────────────────

#[test]
fn search_sessions_by_title() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "s1", Some("Rust 学习笔记"), None, None).unwrap();
    SessionRepo::create(&conn, "s2", Some("Python 项目"), None, None).unwrap();
    SessionRepo::create(&conn, "s3", Some("Rust 编译优化"), None, None).unwrap();

    let results = SessionRepo::search(&conn, "Rust").unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn search_sessions_by_project_name() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "s1", None, None, Some("/code/misaka-tauri")).unwrap();
    SessionRepo::create(&conn, "s2", None, None, Some("/code/other-project")).unwrap();

    let results = SessionRepo::search(&conn, "misaka").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "s1");
}

#[test]
fn search_sessions_excludes_archived() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "s1", Some("Archived Match"), None, None).unwrap();
    SessionRepo::update(&conn, "s1", None, None, None, Some("archived")).unwrap();

    let results = SessionRepo::search(&conn, "Match").unwrap();
    assert_eq!(results.len(), 0);
}

// ─── new fields verification ─────────────────────────────────────────

#[test]
fn session_has_new_fields_with_defaults() {
    let conn = create_test_db();
    let session = SessionRepo::create(&conn, "s-new", None, None, None).unwrap();

    assert_eq!(session.total_input_tokens, 0);
    assert_eq!(session.total_output_tokens, 0);
    assert!(session.last_message_at.is_none());
    assert!(!session.pinned);
    assert!(session.group_name.is_none());
}

// ─── pin_session ────────────────────────────────────────────────────

#[test]
fn pin_session_sets_pinned_true() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "pin-1", Some("Test"), None, None).unwrap();

    SessionRepo::pin_session(&conn, "pin-1", true).unwrap();

    let s = SessionRepo::find_by_id(&conn, "pin-1").unwrap();
    assert!(s.pinned);
}

#[test]
fn pin_session_unpin() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "pin-2", None, None, None).unwrap();
    SessionRepo::pin_session(&conn, "pin-2", true).unwrap();
    SessionRepo::pin_session(&conn, "pin-2", false).unwrap();

    let s = SessionRepo::find_by_id(&conn, "pin-2").unwrap();
    assert!(!s.pinned);
}

// ─── archive_session / unarchive_session ────────────────────────────

#[test]
fn archive_and_unarchive_session() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "arc-1", Some("To Archive"), None, None).unwrap();

    SessionRepo::archive_session(&conn, "arc-1").unwrap();
    let s = SessionRepo::find_by_id(&conn, "arc-1").unwrap();
    assert_eq!(s.status, "archived");

    let active = SessionRepo::list(&conn, Some("active")).unwrap();
    assert!(active.iter().all(|s| s.id != "arc-1"));

    let archived = SessionRepo::list(&conn, Some("archived")).unwrap();
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].id, "arc-1");

    SessionRepo::unarchive_session(&conn, "arc-1").unwrap();
    let s = SessionRepo::find_by_id(&conn, "arc-1").unwrap();
    assert_eq!(s.status, "active");
}

// ─── set_group / list_groups ────────────────────────────────────────

#[test]
fn set_group_and_list_groups() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "grp-1", Some("A"), None, None).unwrap();
    SessionRepo::create(&conn, "grp-2", Some("B"), None, None).unwrap();
    SessionRepo::create(&conn, "grp-3", Some("C"), None, None).unwrap();

    SessionRepo::set_group(&conn, "grp-1", Some("Work")).unwrap();
    SessionRepo::set_group(&conn, "grp-2", Some("Personal")).unwrap();
    SessionRepo::set_group(&conn, "grp-3", Some("Work")).unwrap();

    let s = SessionRepo::find_by_id(&conn, "grp-1").unwrap();
    assert_eq!(s.group_name, Some("Work".to_string()));

    let groups = SessionRepo::list_groups(&conn).unwrap();
    assert_eq!(groups.len(), 2);
    assert!(groups.contains(&"Work".to_string()));
    assert!(groups.contains(&"Personal".to_string()));
}

#[test]
fn set_group_to_none_removes_group() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "grp-rm", None, None, None).unwrap();
    SessionRepo::set_group(&conn, "grp-rm", Some("Test")).unwrap();

    let s = SessionRepo::find_by_id(&conn, "grp-rm").unwrap();
    assert_eq!(s.group_name, Some("Test".to_string()));

    SessionRepo::set_group(&conn, "grp-rm", None).unwrap();

    let s = SessionRepo::find_by_id(&conn, "grp-rm").unwrap();
    assert!(s.group_name.is_none());
}

#[test]
fn list_groups_excludes_archived() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "grp-arc", None, None, None).unwrap();
    SessionRepo::set_group(&conn, "grp-arc", Some("Archived Group")).unwrap();
    SessionRepo::archive_session(&conn, "grp-arc").unwrap();

    let groups = SessionRepo::list_groups(&conn).unwrap();
    assert!(!groups.contains(&"Archived Group".to_string()));
}

// ─── list_all_for_export ────────────────────────────────────────────

#[test]
fn list_all_for_export_includes_all_statuses() {
    let conn = create_test_db();
    SessionRepo::create(&conn, "exp-1", Some("Active"), None, None).unwrap();
    SessionRepo::create(&conn, "exp-2", Some("Archived"), None, None).unwrap();
    SessionRepo::archive_session(&conn, "exp-2").unwrap();

    let all = SessionRepo::list_all_for_export(&conn).unwrap();
    assert_eq!(all.len(), 2);
}
