use rusqlite::Connection;
use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::{McpServerRecord, McpServerRepo};

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    conn
}

#[test]
fn test_migration_v3_creates_mcp_servers_table() {
    let conn = create_test_db();

    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM _schema_version", [], |row| row.get(0))
        .unwrap();
    assert!(version >= 3, "Expected schema version >= 3, got {}", version);

    conn.execute(
        "INSERT INTO mcp_servers (id, name, transport_json) VALUES ('t1', 'Test', '{}')",
        [],
    )
    .unwrap();
}

#[test]
fn test_insert_and_find_by_id() {
    let conn = create_test_db();

    let record = McpServerRecord {
        id: "srv-1".to_string(),
        name: "Filesystem".to_string(),
        transport_json: r#"{"type":"stdio","command":"npx","args":["-y","server-fs"]}"#.to_string(),
        auto_connect: true,
        env_json: Some(r#"{"HOME":"/tmp"}"#.to_string()),
        created_at: String::new(),
        updated_at: String::new(),
    };

    McpServerRepo::insert(&conn, &record).unwrap();

    let found = McpServerRepo::find_by_id(&conn, "srv-1").unwrap();
    assert_eq!(found.name, "Filesystem");
    assert!(found.auto_connect);
    assert!(found.env_json.is_some());
}

#[test]
fn test_list_all() {
    let conn = create_test_db();

    for i in 0..3 {
        let record = McpServerRecord {
            id: format!("srv-{}", i),
            name: format!("Server {}", i),
            transport_json: "{}".to_string(),
            auto_connect: i % 2 == 0,
            env_json: None,
            created_at: String::new(),
            updated_at: String::new(),
        };
        McpServerRepo::insert(&conn, &record).unwrap();
    }

    let all = McpServerRepo::list_all(&conn).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_update() {
    let conn = create_test_db();

    let record = McpServerRecord {
        id: "srv-u".to_string(),
        name: "Before".to_string(),
        transport_json: "{}".to_string(),
        auto_connect: false,
        env_json: None,
        created_at: String::new(),
        updated_at: String::new(),
    };
    McpServerRepo::insert(&conn, &record).unwrap();

    let updated = McpServerRecord {
        id: "srv-u".to_string(),
        name: "After".to_string(),
        transport_json: r#"{"type":"http","url":"http://x"}"#.to_string(),
        auto_connect: true,
        env_json: Some(r#"{"KEY":"val"}"#.to_string()),
        created_at: String::new(),
        updated_at: String::new(),
    };
    McpServerRepo::update(&conn, &updated).unwrap();

    let found = McpServerRepo::find_by_id(&conn, "srv-u").unwrap();
    assert_eq!(found.name, "After");
    assert!(found.auto_connect);
    assert!(found.env_json.is_some());
}

#[test]
fn test_delete() {
    let conn = create_test_db();

    let record = McpServerRecord {
        id: "srv-d".to_string(),
        name: "ToDelete".to_string(),
        transport_json: "{}".to_string(),
        auto_connect: true,
        env_json: None,
        created_at: String::new(),
        updated_at: String::new(),
    };
    McpServerRepo::insert(&conn, &record).unwrap();

    McpServerRepo::delete(&conn, "srv-d").unwrap();

    let result = McpServerRepo::find_by_id(&conn, "srv-d");
    assert!(result.is_err());
}

#[test]
fn test_find_nonexistent() {
    let conn = create_test_db();
    let result = McpServerRepo::find_by_id(&conn, "does-not-exist");
    assert!(result.is_err());
}
