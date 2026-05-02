#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    misaka_x_lib::run();
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn test_init_database() {
        let db_path = PathBuf::from("test.db");

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
                .query_row("SELECT version FROM _schema_version", [], |row| row.get(0))
                .unwrap();
            assert_eq!(version, 1);

            tables
        };

        assert!(tables.contains(&"_schema_version".to_string()));
        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"settings".to_string()));
        assert!(tables.contains(&"router_configs".to_string()));
        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"knowledge_docs".to_string()));

        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(PathBuf::from("test.db-wal"));
        let _ = std::fs::remove_file(PathBuf::from("test.db-shm"));
    }
}
