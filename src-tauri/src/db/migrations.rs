use anyhow::Result;
use rusqlite::Connection;

/// Run all pending schema migrations.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create schema version table if not exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _schema_version (
            version INTEGER PRIMARY KEY
        );",
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        migrate_v1(conn)?;
    }

    if current_version < 2 {
        migrate_v2(conn)?;
    }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Settings table (key-value store)
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Sessions table
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT,
            model TEXT,
            system_prompt TEXT,
            working_directory TEXT,
            project_name TEXT,
            status TEXT DEFAULT 'active',
            mode TEXT DEFAULT 'agent',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Messages table
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            token_usage TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
        );

        -- Router configs (API keys, providers)
        CREATE TABLE IF NOT EXISTS router_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider TEXT NOT NULL,
            api_key_encrypted TEXT,
            model TEXT,
            base_url TEXT,
            config_json TEXT,
            is_active INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Tasks table
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            parent_task_id TEXT,
            title TEXT NOT NULL,
            status TEXT DEFAULT 'pending',
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL,
            FOREIGN KEY (parent_task_id) REFERENCES tasks(id) ON DELETE SET NULL
        );

        -- Knowledge documents metadata
        CREATE TABLE IF NOT EXISTS knowledge_docs (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            source_path TEXT,
            file_type TEXT,
            chunk_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Full-text search index for messages
        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts
        USING fts5(content, session_id, role);

        -- Full-text search index for knowledge base
        CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_fts
        USING fts5(title, content, source_path);

        -- Update schema version
        INSERT INTO _schema_version (version) VALUES (1);
        ",
    )?;

    tracing::info!("Database migrated to version 1");
    Ok(())
}

fn migrate_v2(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- messages 表增强
        ALTER TABLE messages ADD COLUMN model TEXT;
        ALTER TABLE messages ADD COLUMN thinking_content TEXT;
        ALTER TABLE messages ADD COLUMN attachments TEXT;
        ALTER TABLE messages ADD COLUMN status TEXT DEFAULT 'complete';

        -- sessions 表增强
        ALTER TABLE sessions ADD COLUMN total_input_tokens INTEGER DEFAULT 0;
        ALTER TABLE sessions ADD COLUMN total_output_tokens INTEGER DEFAULT 0;
        ALTER TABLE sessions ADD COLUMN last_message_at DATETIME;
        ALTER TABLE sessions ADD COLUMN pinned INTEGER DEFAULT 0;
        ALTER TABLE sessions ADD COLUMN group_name TEXT;
        ALTER TABLE sessions ADD COLUMN working_dir_remote TEXT;
        ALTER TABLE sessions ADD COLUMN working_dir_remote_type TEXT;

        -- router_configs 新增接口兼容性字段
        ALTER TABLE router_configs ADD COLUMN api_compat TEXT DEFAULT NULL;

        -- 用户自定义模型表
        CREATE TABLE IF NOT EXISTS custom_models (
            id TEXT PRIMARY KEY,
            router_config_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            display_name TEXT NOT NULL,
            supports_vision INTEGER DEFAULT 0,
            supports_thinking INTEGER DEFAULT 0,
            max_tokens INTEGER,
            context_window INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (router_config_id) REFERENCES router_configs(id) ON DELETE CASCADE,
            UNIQUE (router_config_id, model_id)
        );

        -- 最近使用的工作目录
        CREATE TABLE IF NOT EXISTS recent_directories (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            display_name TEXT,
            last_used_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            use_count INTEGER DEFAULT 1
        );

        -- 更新 schema 版本
        INSERT INTO _schema_version (version) VALUES (2);
        ",
    )?;

    tracing::info!("Database migrated to version 2");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        conn
    }

    #[test]
    fn test_migration_v1() {
        let conn = create_test_db();
        run_migrations(&conn).unwrap();

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM _schema_version", [], |row| row.get(0))
            .unwrap();
        assert!(version >= 1);

        conn.execute(
            "INSERT INTO sessions (id, title) VALUES ('s1', 'Test Session')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content) VALUES ('m1', 's1', 'user', 'hello')",
            [],
        )
        .unwrap();
    }

    #[test]
    fn test_migration_v2_adds_custom_models_table() {
        let conn = create_test_db();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO router_configs (id, name, provider) VALUES ('rc1', 'Test', 'openai')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO custom_models (id, router_config_id, model_id, display_name)
             VALUES ('cm1', 'rc1', 'gpt-4o-ft', 'Fine-tuned')",
            [],
        )
        .unwrap();

        let display_name: String = conn
            .query_row(
                "SELECT display_name FROM custom_models WHERE id = 'cm1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(display_name, "Fine-tuned");
    }

    #[test]
    fn test_migration_v2_adds_api_compat_column() {
        let conn = create_test_db();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO router_configs (id, name, provider, api_compat)
             VALUES ('rc1', 'DeepSeek', 'deepseek', 'openai')",
            [],
        )
        .unwrap();

        let compat: Option<String> = conn
            .query_row(
                "SELECT api_compat FROM router_configs WHERE id = 'rc1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(compat, Some("openai".to_string()));
    }

    #[test]
    fn test_migration_v2_adds_session_token_columns() {
        let conn = create_test_db();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO sessions (id, title, total_input_tokens, total_output_tokens)
             VALUES ('s1', 'Test', 100, 200)",
            [],
        )
        .unwrap();

        let (input, output): (i32, i32) = conn
            .query_row(
                "SELECT total_input_tokens, total_output_tokens FROM sessions WHERE id = 's1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(input, 100);
        assert_eq!(output, 200);
    }

    #[test]
    fn test_migration_v2_adds_message_columns() {
        let conn = create_test_db();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO sessions (id) VALUES ('s1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, model, thinking_content, status)
             VALUES ('m1', 's1', 'assistant', 'Hello!', 'gpt-4o', 'I need to greet the user.', 'complete')",
            [],
        )
        .unwrap();

        let (model, thinking, status): (Option<String>, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT model, thinking_content, status FROM messages WHERE id = 'm1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(model, Some("gpt-4o".to_string()));
        assert_eq!(thinking, Some("I need to greet the user.".to_string()));
        assert_eq!(status, Some("complete".to_string()));
    }

    #[test]
    fn test_migration_v2_creates_recent_directories() {
        let conn = create_test_db();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO recent_directories (id, path, display_name, use_count)
             VALUES ('rd1', '/home/user/project', 'project', 3)",
            [],
        )
        .unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT use_count FROM recent_directories WHERE id = 'rd1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = create_test_db();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM _schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_custom_models_cascade_delete() {
        let conn = create_test_db();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO router_configs (id, name, provider) VALUES ('rc1', 'Test', 'openai')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO custom_models (id, router_config_id, model_id, display_name)
             VALUES ('cm1', 'rc1', 'model-1', 'Model 1')",
            [],
        )
        .unwrap();

        conn.execute("DELETE FROM router_configs WHERE id = 'rc1'", [])
            .unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM custom_models WHERE router_config_id = 'rc1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "Custom models should be cascade deleted");
    }
}
