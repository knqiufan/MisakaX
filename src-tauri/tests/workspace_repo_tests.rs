use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::WorkspaceRepo;
use rusqlite::Connection;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    conn
}

// ─── find_recent ─────────────────────────────────────────────────────

#[test]
fn find_recent_returns_empty_when_no_data() {
    let conn = create_test_db();
    let result = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert!(result.is_empty());
}

#[test]
fn find_recent_returns_inserted_directories() {
    let conn = create_test_db();

    conn.execute(
        "INSERT INTO recent_directories (id, path, display_name, last_used_at, use_count)
         VALUES ('id1', '/home/user/project-a', 'project-a', '2025-01-01 10:00:00', 1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO recent_directories (id, path, display_name, last_used_at, use_count)
         VALUES ('id2', '/home/user/project-b', 'project-b', '2025-01-01 11:00:00', 1)",
        [],
    )
    .unwrap();

    let dirs = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert_eq!(dirs.len(), 2);
    assert_eq!(dirs[0].path, "/home/user/project-b");
    assert_eq!(dirs[1].path, "/home/user/project-a");
}

#[test]
fn find_recent_respects_limit() {
    let conn = create_test_db();

    for i in 0..5 {
        let id = format!("id-{}", i);
        let path = format!("/path/project-{}", i);
        WorkspaceRepo::record_usage(&conn, &id, &path, None).unwrap();
    }

    let dirs = WorkspaceRepo::find_recent(&conn, 3).unwrap();
    assert_eq!(dirs.len(), 3);
}

#[test]
fn find_recent_ordered_by_last_used_desc() {
    let conn = create_test_db();

    conn.execute(
        "INSERT INTO recent_directories (id, path, display_name, last_used_at, use_count)
         VALUES ('a', '/path/old', 'old', '2024-01-01 00:00:00', 1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO recent_directories (id, path, display_name, last_used_at, use_count)
         VALUES ('b', '/path/new', 'new', '2025-06-01 00:00:00', 1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO recent_directories (id, path, display_name, last_used_at, use_count)
         VALUES ('c', '/path/mid', 'mid', '2024-06-01 00:00:00', 1)",
        [],
    )
    .unwrap();

    let dirs = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert_eq!(dirs.len(), 3);
    assert_eq!(dirs[0].path, "/path/new");
    assert_eq!(dirs[1].path, "/path/mid");
    assert_eq!(dirs[2].path, "/path/old");
}

// ─── record_usage ─────────────────────────────────────────────────────

#[test]
fn record_usage_inserts_new_directory() {
    let conn = create_test_db();

    WorkspaceRepo::record_usage(&conn, "id1", "/home/user/project", Some("project")).unwrap();

    let dirs = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0].path, "/home/user/project");
    assert_eq!(dirs[0].display_name, Some("project".to_string()));
    assert_eq!(dirs[0].use_count, 1);
}

#[test]
fn record_usage_increments_count_on_duplicate_path() {
    let conn = create_test_db();

    WorkspaceRepo::record_usage(&conn, "id1", "/home/user/project", Some("project")).unwrap();
    WorkspaceRepo::record_usage(&conn, "id2", "/home/user/project", Some("project")).unwrap();
    WorkspaceRepo::record_usage(&conn, "id3", "/home/user/project", Some("project")).unwrap();

    let dirs = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0].use_count, 3);
}

#[test]
fn record_usage_preserves_existing_display_name_when_null() {
    let conn = create_test_db();

    WorkspaceRepo::record_usage(&conn, "id1", "/path/foo", Some("MyProject")).unwrap();
    WorkspaceRepo::record_usage(&conn, "id2", "/path/foo", None).unwrap();

    let dirs = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert_eq!(dirs[0].display_name, Some("MyProject".to_string()));
}

#[test]
fn record_usage_updates_display_name_when_provided() {
    let conn = create_test_db();

    WorkspaceRepo::record_usage(&conn, "id1", "/path/foo", Some("OldName")).unwrap();
    WorkspaceRepo::record_usage(&conn, "id2", "/path/foo", Some("NewName")).unwrap();

    let dirs = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert_eq!(dirs[0].display_name, Some("NewName".to_string()));
}

// ─── delete_by_path ──────────────────────────────────────────────────

#[test]
fn delete_by_path_removes_existing() {
    let conn = create_test_db();

    WorkspaceRepo::record_usage(&conn, "id1", "/path/to/delete", None).unwrap();
    assert!(WorkspaceRepo::exists(&conn, "/path/to/delete").unwrap());

    let deleted = WorkspaceRepo::delete_by_path(&conn, "/path/to/delete").unwrap();
    assert!(deleted);
    assert!(!WorkspaceRepo::exists(&conn, "/path/to/delete").unwrap());
}

#[test]
fn delete_by_path_returns_false_for_nonexistent() {
    let conn = create_test_db();
    let deleted = WorkspaceRepo::delete_by_path(&conn, "/nonexistent").unwrap();
    assert!(!deleted);
}

#[test]
fn delete_by_path_does_not_affect_others() {
    let conn = create_test_db();

    WorkspaceRepo::record_usage(&conn, "id1", "/path/a", None).unwrap();
    WorkspaceRepo::record_usage(&conn, "id2", "/path/b", None).unwrap();

    WorkspaceRepo::delete_by_path(&conn, "/path/a").unwrap();

    let dirs = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0].path, "/path/b");
}

// ─── exists ──────────────────────────────────────────────────────────

#[test]
fn exists_returns_true_for_existing_path() {
    let conn = create_test_db();
    WorkspaceRepo::record_usage(&conn, "id1", "/path/exists", None).unwrap();
    assert!(WorkspaceRepo::exists(&conn, "/path/exists").unwrap());
}

#[test]
fn exists_returns_false_for_missing_path() {
    let conn = create_test_db();
    assert!(!WorkspaceRepo::exists(&conn, "/path/missing").unwrap());
}
