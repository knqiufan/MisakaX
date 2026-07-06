#[test]
fn test_init_database() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_init.db");

    let tables = {
        let conn =
            misaka_x_lib::db::init_database(&db_path).expect("Failed to initialize test database");

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM _schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(
            version >= 3,
            "Expected schema version >= 3, got {}",
            version
        );

        tables
    };

    assert!(tables.contains(&"_schema_version".to_string()));
    assert!(tables.contains(&"sessions".to_string()));
    assert!(tables.contains(&"messages".to_string()));
    assert!(tables.contains(&"settings".to_string()));
    assert!(tables.contains(&"router_configs".to_string()));
    assert!(tables.contains(&"tasks".to_string()));
    assert!(tables.contains(&"knowledge_docs".to_string()));
    assert!(tables.contains(&"mcp_servers".to_string()));
}
