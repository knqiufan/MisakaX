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

    if current_version < 3 {
        migrate_v3(conn)?;
    }

    if current_version < 4 {
        migrate_v4(conn)?;
    }

    if current_version < 5 {
        migrate_v5(conn)?;
    }

    if current_version < 6 {
        migrate_v6(conn)?;
    }

    if current_version < 7 {
        migrate_v7(conn)?;
    }

    if current_version < 8 {
        migrate_v8(conn)?;
    }

    if current_version < 9 {
        migrate_v9(conn)?;
    }

    if current_version < 10 {
        migrate_v10(conn)?;
    }

    if current_version < 11 {
        migrate_v11(conn)?;
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

fn migrate_v3(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            transport_json TEXT NOT NULL,
            auto_connect INTEGER DEFAULT 1,
            env_json TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        INSERT INTO _schema_version (version) VALUES (3);
        ",
    )?;

    tracing::info!("Database migrated to version 3");
    Ok(())
}

fn migrate_v4(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tool_permissions (
            server_id TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            policy TEXT NOT NULL DEFAULT 'ask',
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (server_id, tool_name)
        );

        INSERT INTO _schema_version (version) VALUES (4);
        ",
    )?;

    tracing::info!("Database migrated to version 4");
    Ok(())
}

fn migrate_v5(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        ALTER TABLE messages ADD COLUMN tool_calls TEXT;

        INSERT INTO _schema_version (version) VALUES (5);
        ",
    )?;

    tracing::info!("Database migrated to version 5");
    Ok(())
}

fn migrate_v6(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute_batch(
        "
        ALTER TABLE router_configs ADD COLUMN vendor TEXT;
        ALTER TABLE router_configs ADD COLUMN advanced_json TEXT;

        ALTER TABLE custom_models ADD COLUMN enabled INTEGER DEFAULT 1;
        ALTER TABLE custom_models ADD COLUMN sort_order INTEGER DEFAULT 0;

        UPDATE router_configs
        SET vendor = CASE
            WHEN provider IN ('deepseek', 'zhipu', 'minimax', 'stepfun', 'moonshot', 'siliconflow')
                THEN provider
            ELSE 'custom'
        END
        WHERE vendor IS NULL;

        UPDATE router_configs
        SET provider = CASE
            WHEN provider IN ('openai', 'anthropic', 'google') THEN provider
            WHEN api_compat IN ('openai', 'anthropic', 'google') THEN api_compat
            ELSE 'openai'
        END
        WHERE provider NOT IN ('openai', 'anthropic', 'google');

        ",
    )?;
    inject_builtin_models_for_existing_configs(&tx)?;
    tx.execute("INSERT INTO _schema_version (version) VALUES (6)", [])?;
    tx.commit()?;

    tracing::info!("Database migrated to version 6");
    Ok(())
}

fn migrate_v7(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        ALTER TABLE custom_models ADD COLUMN thinking_off_model_id TEXT;
        ALTER TABLE sessions ADD COLUMN workspace_kind TEXT NOT NULL DEFAULT 'custom';

        INSERT INTO _schema_version (version) VALUES (7);
        ",
    )?;

    tracing::info!("Database migrated to version 7");
    Ok(())
}

fn migrate_v8(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS workspace_preferences (
            workspace_key TEXT PRIMARY KEY,
            pinned INTEGER NOT NULL DEFAULT 0,
            hidden INTEGER NOT NULL DEFAULT 0,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        INSERT INTO _schema_version (version) VALUES (8);
        ",
    )?;

    tracing::info!("Database migrated to version 8");
    Ok(())
}

fn migrate_v9(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        ALTER TABLE custom_models ADD COLUMN model_types_json TEXT NOT NULL DEFAULT '[]';

        INSERT INTO _schema_version (version) VALUES (9);
        ",
    )?;

    tracing::info!("Database migrated to version 9");
    Ok(())
}

fn migrate_v10(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS skills (
            slug TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            version TEXT,
            source_kind TEXT NOT NULL,
            source_ref TEXT,
            source_url TEXT,
            checksum TEXT NOT NULL,
            installed_path TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            health TEXT NOT NULL DEFAULT 'healthy',
            risk_json TEXT NOT NULL DEFAULT '{}',
            installed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS message_skill_selections (
            message_id TEXT NOT NULL,
            skill_slug TEXT NOT NULL,
            sort_order INTEGER NOT NULL,
            PRIMARY KEY (message_id, skill_slug),
            FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_skills_enabled
        ON skills(enabled, health, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_message_skill_selections_message
        ON message_skill_selections(message_id, sort_order);

        INSERT INTO _schema_version (version) VALUES (10);
        ",
    )?;

    tracing::info!("Database migrated to version 10");
    Ok(())
}

fn migrate_v11(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute_batch(
        "
        CREATE TABLE skill_sources (
            skill_id TEXT PRIMARY KEY,
            slug TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            version TEXT,
            source_kind TEXT NOT NULL,
            source_locator TEXT NOT NULL,
            source_ref TEXT,
            source_url TEXT,
            artifact_hash TEXT NOT NULL,
            installed_path TEXT NOT NULL,
            is_managed INTEGER NOT NULL DEFAULT 0,
            user_enabled INTEGER NOT NULL DEFAULT 0,
            health TEXT NOT NULL DEFAULT 'healthy',
            security_state TEXT NOT NULL DEFAULT 'pending_user',
            effective_rank INTEGER NOT NULL DEFAULT 0,
            disabled_reason TEXT,
            risk_json TEXT NOT NULL DEFAULT '{}',
            installed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(source_kind, source_locator)
        );

        INSERT INTO skill_sources (
            skill_id, slug, name, description, version, source_kind, source_locator,
            source_ref, source_url, artifact_hash, installed_path, is_managed,
            user_enabled, health, security_state, effective_rank, risk_json,
            installed_at, updated_at
        )
        SELECT
            lower(hex(randomblob(16))), slug, name, description, version,
            'managed', installed_path, source_ref, source_url, checksum,
            installed_path, 1, enabled, health, 'legacy_allowed', 400,
            risk_json, installed_at, updated_at
        FROM skills;

        CREATE TABLE message_skill_selections_v11 (
            message_id TEXT NOT NULL,
            skill_id TEXT NOT NULL,
            skill_slug_snapshot TEXT NOT NULL,
            artifact_hash_snapshot TEXT NOT NULL,
            sort_order INTEGER NOT NULL,
            PRIMARY KEY (message_id, skill_id),
            FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
        );

        INSERT INTO message_skill_selections_v11 (
            message_id, skill_id, skill_slug_snapshot, artifact_hash_snapshot, sort_order
        )
        SELECT selection.message_id, source.skill_id, selection.skill_slug,
               source.artifact_hash, selection.sort_order
        FROM message_skill_selections selection
        JOIN skill_sources source
          ON source.slug = selection.skill_slug AND source.is_managed = 1;

        DROP TABLE message_skill_selections;
        ALTER TABLE message_skill_selections_v11 RENAME TO message_skill_selections;

        CREATE TABLE skill_activation_state (
            singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
            generation INTEGER NOT NULL CHECK(generation > 0),
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        INSERT INTO skill_activation_state(singleton, generation) VALUES (1, 1);

        CREATE INDEX idx_skill_sources_slug_rank
        ON skill_sources(slug, effective_rank DESC, updated_at DESC);
        CREATE INDEX idx_skill_sources_activation
        ON skill_sources(user_enabled, health, security_state, effective_rank DESC);
        CREATE INDEX idx_message_skill_selections_message
        ON message_skill_selections(message_id, sort_order);

        INSERT INTO _schema_version (version) VALUES (11);
        ",
    )?;
    tx.commit()?;

    tracing::info!("Database migrated to version 11");
    Ok(())
}

fn inject_builtin_models_for_existing_configs(conn: &Connection) -> Result<()> {
    let configs = list_router_configs_for_model_injection(conn)?;
    for (router_config_id, provider) in configs {
        let models = crate::services::llm::ModelRegistry::builtin_models(&provider);
        for (index, model) in models.iter().enumerate() {
            insert_builtin_model(conn, &router_config_id, model, index as i32)?;
        }
    }
    Ok(())
}

fn list_router_configs_for_model_injection(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT id, provider FROM router_configs")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn insert_builtin_model(
    conn: &Connection,
    router_config_id: &str,
    model: &crate::services::llm::ModelInfo,
    sort_order: i32,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO custom_models (
            id, router_config_id, model_id, display_name, supports_vision,
            supports_thinking, max_tokens, context_window, enabled, sort_order
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            router_config_id,
            model.model_id,
            model.display_name,
            model.supports_vision as i32,
            model.supports_thinking as i32,
            model.max_tokens,
            model.context_window,
            sort_order,
        ],
    )?;
    Ok(())
}
