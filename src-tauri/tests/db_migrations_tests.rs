use misaka_x_lib::db::{backup_before_migration, migrations::run_migrations};
use rusqlite::Connection;

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
        .query_row("SELECT MAX(version) FROM _schema_version", [], |row| {
            row.get(0)
        })
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

    conn.execute("INSERT INTO sessions (id) VALUES ('s1')", [])
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
        .query_row("SELECT MAX(version) FROM _schema_version", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(version, 15);
}

#[test]
fn test_v15_creates_profile_and_usage_ledger_with_constraints() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO user_profiles (
            profile_id, display_name, profile_kind, timezone_mode, week_start
         ) VALUES ('profile-v15', 'Local User', 'local', 'system', 1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO llm_usage_events (
            event_id, profile_id, operation_key, measurement_key, operation_kind,
            input_tokens, output_tokens, total_tokens, measurement_source, outcome,
            counts_toward_totals, counts_toward_activity, counts_toward_trend,
            occurred_at_utc, local_date, utc_offset_minutes, metadata_json
         ) VALUES (
            'event-v15', 'profile-v15', 'assistant:m1', 'assistant:m1:model:a', 'chat',
            10, 5, 15, 'provider_reported', 'completed', 1, 1, 1,
            '2026-08-13T00:00:00Z', '2026-08-13', 480, '{}'
         )",
        [],
    )
    .unwrap();

    let invalid_negative = conn.execute(
        "UPDATE llm_usage_events SET total_tokens = -1 WHERE event_id = 'event-v15'",
        [],
    );
    let invalid_source = conn.execute(
        "UPDATE llm_usage_events SET measurement_source = 'guessed' WHERE event_id = 'event-v15'",
        [],
    );
    let invalid_flag = conn.execute(
        "UPDATE llm_usage_events SET counts_toward_totals = 2 WHERE event_id = 'event-v15'",
        [],
    );
    assert!(invalid_negative.is_err());
    assert!(invalid_source.is_err());
    assert!(invalid_flag.is_err());

    let index_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'index' AND name IN (
                'idx_usage_profile_date', 'idx_usage_profile_model_date',
                'idx_usage_operation', 'idx_usage_session', 'idx_usage_import_source'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(index_count, 5);
}

#[test]
fn test_v15_upgrade_preserves_v14_data_and_backup_is_restorable() {
    let temporary = tempfile::tempdir().unwrap();
    let db_path = temporary.path().join("misaka.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations_to_v14(&conn);
    conn.execute(
        "INSERT INTO sessions (
            id, title, total_input_tokens, total_output_tokens
         ) VALUES ('session-v14', 'Before v15', 100, 50)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO messages (
            id, session_id, role, content, token_usage, model
         ) VALUES (
            'message-v14', 'session-v14', 'assistant', 'kept',
            '{\"input_tokens\":100,\"output_tokens\":50,\"total_tokens\":150}',
            'model-v14'
         )",
        [],
    )
    .unwrap();

    let backup = backup_before_migration(&conn, &db_path, 15)
        .unwrap()
        .expect("v14 database should be backed up");
    assert!(backup.ends_with("misaka.pre-v15.sqlite3"));
    run_migrations(&conn).unwrap();
    run_migrations(&conn).unwrap();

    let preserved: (String, String, i64, i64) = conn
        .query_row(
            "SELECT messages.content, messages.model,
                    sessions.total_input_tokens, sessions.total_output_tokens
             FROM messages JOIN sessions ON sessions.id = messages.session_id
             WHERE messages.id = 'message-v14'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(preserved, ("kept".into(), "model-v14".into(), 100, 50));
    drop(conn);

    let restored = Connection::open(backup).unwrap();
    let restored_version: i64 = restored
        .query_row("SELECT MAX(version) FROM _schema_version", [], |row| {
            row.get(0)
        })
        .unwrap();
    let v15_tables: i64 = restored
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'table' AND name IN ('user_profiles', 'llm_usage_events')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(restored_version, 14);
    assert_eq!(v15_tables, 0);
}

#[test]
fn test_v11_skill_fixture_survives_forward_idempotent_migration_and_rollback() {
    let conn = create_test_db();
    run_migrations_to_v10(&conn);
    conn.execute("INSERT INTO sessions (id) VALUES ('skill-session')", [])
        .unwrap();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content)
         VALUES ('skill-message', 'skill-session', 'user', 'use the skill')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO skills (
            slug, name, description, source_kind, checksum, installed_path,
            enabled, health, risk_json
         ) VALUES (
            'legacy-skill', 'Legacy Skill', 'Fixture', 'local', 'abc123',
            'C:/fixture/legacy-skill', 1, 'healthy', '{}'
         )",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO message_skill_selections (message_id, skill_slug, sort_order)
         VALUES ('skill-message', 'legacy-skill', 0)",
        [],
    )
    .unwrap();

    run_migrations(&conn).unwrap();
    let stable_id: String = conn
        .query_row(
            "SELECT skill_id FROM skill_sources WHERE slug = 'legacy-skill'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!stable_id.is_empty());
    run_migrations(&conn).unwrap();
    conn.execute_batch(
        "BEGIN IMMEDIATE;
         UPDATE skill_sources SET user_enabled = 0 WHERE slug = 'legacy-skill';
         ROLLBACK;",
    )
    .unwrap();

    let (enabled, security_state): (i64, String) = conn
        .query_row(
            "SELECT user_enabled, security_state FROM skill_sources
             WHERE slug = 'legacy-skill'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let (selected_id, slug_snapshot, hash_snapshot): (String, String, String) = conn
        .query_row(
            "SELECT skill_id, skill_slug_snapshot, artifact_hash_snapshot
             FROM message_skill_selections
             WHERE message_id = 'skill-message'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(enabled, 0);
    assert_eq!(security_state, "unscanned");
    assert_eq!(selected_id, stable_id);
    assert_eq!(slug_snapshot, "legacy-skill");
    assert_eq!(hash_snapshot, "abc123");
}

#[test]
fn test_v12_security_schema_enforces_scan_and_finding_contracts() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();
    conn.execute(
        "INSERT INTO skill_artifacts (
            artifact_hash, size_bytes, source_json, quarantine_path, state
         ) VALUES ('hash-v12', 12, '{}', 'C:/quarantine/hash-v12', 'quarantined')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO skill_scan_runs (
            scan_id, artifact_hash, state, engine_versions_json,
            policy_version, correlation_id
         ) VALUES ('scan-v12', 'hash-v12', 'queued', '{}', 'balanced-v1', 'corr-v12')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO skill_findings (
            finding_id, scan_id, engine, rule_id, severity, category,
            title, detail, fingerprint
         ) VALUES (
            'finding-v12', 'scan-v12', 'builtin', 'RULE-1', 'high',
            'command_execution', 'Dangerous command', 'Redacted detail', 'fp-v12'
         )",
        [],
    )
    .unwrap();

    let invalid = conn.execute(
        "UPDATE skill_scan_runs SET state = 'made-up' WHERE scan_id = 'scan-v12'",
        [],
    );
    assert!(invalid.is_err());
    let scan_column: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('skill_sources') WHERE name = 'current_scan_id'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(scan_column, 1);
}

#[test]
fn test_v12_upgrade_backup_restores_v11_without_security_tables() {
    let temporary = tempfile::tempdir().unwrap();
    let db_path = temporary.path().join("misaka.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations_to_v11(&conn);

    let backup = backup_before_migration(&conn, &db_path, 12)
        .unwrap()
        .expect("v11 database should be backed up");
    run_migrations(&conn).unwrap();
    drop(conn);

    let restored = Connection::open(backup).unwrap();
    let version: i64 = restored
        .query_row("SELECT MAX(version) FROM _schema_version", [], |row| {
            row.get(0)
        })
        .unwrap();
    let security_tables: i64 = restored
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'table' AND name IN (
                'skill_artifacts', 'skill_scan_runs', 'skill_findings', 'skill_approvals'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, 11);
    assert_eq!(security_tables, 0);
}

#[test]
fn test_v13_migrates_legacy_sources_and_preserves_a_restorable_v12_backup() {
    let temporary = tempfile::tempdir().unwrap();
    let db_path = temporary.path().join("misaka.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations_to_v12(&conn);
    conn.execute(
        "INSERT INTO skills (
            slug, name, description, source_kind, checksum, installed_path,
            enabled, health, risk_json
         ) VALUES ('legacy-v13', 'Legacy', 'Fixture', 'local', 'hash-v13',
                   'C:/legacy-v13', 1, 'healthy', '{\"has_scripts\":true}')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO skill_sources (
            skill_id, slug, name, description, source_kind, source_locator,
            artifact_hash, installed_path, is_managed, user_enabled, health,
            security_state, effective_rank, risk_json
         ) VALUES ('skill-v13', 'legacy-v13', 'Legacy', 'Fixture', 'managed',
                   'C:/legacy-v13', 'hash-v13', 'C:/legacy-v13', 1, 1,
                   'healthy', 'legacy_allowed', 400, '{}')",
        [],
    )
    .unwrap();

    let backup = backup_before_migration(&conn, &db_path, 13)
        .unwrap()
        .expect("v12 database should be backed up");
    run_migrations(&conn).unwrap();

    let migrated: (i64, String, String) = conn
        .query_row(
            "SELECT user_enabled, security_state, disabled_reason
             FROM skill_sources WHERE skill_id = 'skill-v13'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        migrated,
        (
            0,
            "unscanned".to_string(),
            "migration_scan_required".to_string()
        )
    );
    let old_tables: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'skills'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let old_risk_column: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('skill_sources') WHERE name = 'risk_json'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(old_tables, 0);
    assert_eq!(old_risk_column, 0);
    drop(conn);

    let restored = Connection::open(backup).unwrap();
    let version: i64 = restored
        .query_row("SELECT MAX(version) FROM _schema_version", [], |row| {
            row.get(0)
        })
        .unwrap();
    let legacy_enabled: i64 = restored
        .query_row(
            "SELECT user_enabled FROM skill_sources WHERE skill_id = 'skill-v13'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, 12);
    assert_eq!(legacy_enabled, 1);
}

#[test]
fn test_v11_upgrade_creates_a_restorable_v10_backup() {
    let temporary = tempfile::tempdir().unwrap();
    let db_path = temporary.path().join("misaka.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations_to_v10(&conn);
    conn.execute(
        "INSERT INTO skills (
            slug, name, description, source_kind, checksum, installed_path
         ) VALUES ('backup-skill', 'Backup', 'Fixture', 'local', 'hash', 'C:/backup')",
        [],
    )
    .unwrap();

    let backup = backup_before_migration(&conn, &db_path, 11)
        .unwrap()
        .expect("v10 database should be backed up");
    run_migrations(&conn).unwrap();
    drop(conn);

    let restored = Connection::open(backup).unwrap();
    let version: i64 = restored
        .query_row("SELECT MAX(version) FROM _schema_version", [], |row| {
            row.get(0)
        })
        .unwrap();
    let slug: String = restored
        .query_row(
            "SELECT slug FROM skills WHERE slug = 'backup-skill'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, 10);
    assert_eq!(slug, "backup-skill");
}

#[test]
fn test_migration_v8_creates_workspace_preferences() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO workspace_preferences (workspace_key, pinned, hidden)
         VALUES ('d:/code/misaka-tauri', 1, 1)",
        [],
    )
    .unwrap();

    let (pinned, hidden): (i64, i64) = conn
        .query_row(
            "SELECT pinned, hidden FROM workspace_preferences
             WHERE workspace_key = 'd:/code/misaka-tauri'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!((pinned, hidden), (1, 1));
}

#[test]
fn test_migration_v9_adds_model_types() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO router_configs (id, name, provider) VALUES ('rc-v9', 'OpenAI', 'openai')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO custom_models (id, router_config_id, model_id, display_name, model_types_json)
         VALUES ('cm-v9', 'rc-v9', 'text-embedding-3-small', 'Embedding', '[\"embedding\"]')",
        [],
    )
    .unwrap();
    let model_types: String = conn
        .query_row(
            "SELECT model_types_json FROM custom_models WHERE id = 'cm-v9'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(model_types, "[\"embedding\"]");
}

#[test]
fn test_migration_v6_adds_provider_dialog_columns() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO router_configs (id, name, provider, vendor, advanced_json)
         VALUES ('rc-v6', 'Zhipu', 'openai', 'zhipu', '{\"temperature\":0.7}')",
        [],
    )
    .unwrap();

    let (vendor, advanced_json): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT vendor, advanced_json FROM router_configs WHERE id = 'rc-v6'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(vendor, Some("zhipu".to_string()));
    assert_eq!(advanced_json, Some("{\"temperature\":0.7}".to_string()));
}

#[test]
fn test_migration_v6_adds_custom_model_state_columns() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO router_configs (id, name, provider) VALUES ('rc-v6', 'OpenAI', 'openai')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO custom_models (id, router_config_id, model_id, display_name)
         VALUES ('cm-v6', 'rc-v6', 'gpt-4o', 'GPT-4o')",
        [],
    )
    .unwrap();

    let (enabled, sort_order): (i32, i32) = conn
        .query_row(
            "SELECT enabled, sort_order FROM custom_models WHERE id = 'cm-v6'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(enabled, 1);
    assert_eq!(sort_order, 0);
}

#[test]
fn test_migration_v6_normalizes_legacy_vendor_provider_contract() {
    let conn = create_test_db();
    run_migrations_to_v5(&conn);

    conn.execute(
        "INSERT INTO router_configs (id, name, provider, api_compat)
         VALUES ('rc-deepseek', 'DeepSeek', 'deepseek', 'openai')",
        [],
    )
    .unwrap();

    run_migrations(&conn).unwrap();

    let (provider, vendor): (String, Option<String>) = conn
        .query_row(
            "SELECT provider, vendor FROM router_configs WHERE id = 'rc-deepseek'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(provider, "openai");
    assert_eq!(vendor, Some("deepseek".to_string()));
}

#[test]
fn test_migration_v6_injects_builtin_models_for_existing_router_configs() {
    let conn = create_test_db();
    run_migrations_to_v5(&conn);

    conn.execute(
        "INSERT INTO router_configs (id, name, provider)
         VALUES ('rc-openai', 'OpenAI', 'openai')",
        [],
    )
    .unwrap();

    run_migrations(&conn).unwrap();

    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM custom_models WHERE router_config_id = 'rc-openai'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let enabled_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM custom_models
             WHERE router_config_id = 'rc-openai' AND enabled = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(count > 0);
    assert_eq!(count, enabled_count);
}

fn run_migrations_to_v5(conn: &Connection) {
    run_migrations(conn).unwrap();
    revert_v15(conn);
    revert_v14(conn);
    conn.execute_batch(
        "DROP TABLE skill_security_migration_items;
         DROP TABLE skill_security_migration;
         DROP TABLE skill_approvals;
         DROP TABLE skill_findings;
         DROP TABLE skill_scan_runs;
         DROP TABLE skill_artifacts;
         DROP TABLE message_skill_selections;
         DROP TABLE skill_activation_state;
         DROP TABLE skill_sources;",
    )
    .unwrap();
    conn.execute("DELETE FROM _schema_version WHERE version >= 6", [])
        .unwrap();
    conn.execute("DROP TABLE workspace_preferences", [])
        .unwrap();
    conn.execute("ALTER TABLE custom_models DROP COLUMN model_types_json", [])
        .unwrap();
    conn.execute(
        "ALTER TABLE custom_models DROP COLUMN thinking_off_model_id",
        [],
    )
    .unwrap();
    conn.execute("ALTER TABLE sessions DROP COLUMN workspace_kind", [])
        .unwrap();
    conn.execute("ALTER TABLE router_configs DROP COLUMN vendor", [])
        .unwrap();
    conn.execute("ALTER TABLE router_configs DROP COLUMN advanced_json", [])
        .unwrap();
    conn.execute("ALTER TABLE custom_models DROP COLUMN enabled", [])
        .unwrap();
    conn.execute("ALTER TABLE custom_models DROP COLUMN sort_order", [])
        .unwrap();
}

fn run_migrations_to_v10(conn: &Connection) {
    run_migrations(conn).unwrap();
    revert_v15(conn);
    revert_v14(conn);
    revert_v13(conn);
    conn.execute_batch(
        "DROP TABLE skill_approvals;
         DROP TABLE skill_findings;
         DROP TABLE skill_scan_runs;
         DROP TABLE skill_artifacts;
         DROP INDEX idx_skill_sources_current_scan;
         ALTER TABLE skill_sources DROP COLUMN current_scan_id;
         DELETE FROM _schema_version WHERE version = 12;
         DROP TABLE message_skill_selections;
         DROP TABLE skill_activation_state;
         DROP TABLE skill_sources;
         CREATE TABLE message_skill_selections (
             message_id TEXT NOT NULL,
             skill_slug TEXT NOT NULL,
             sort_order INTEGER NOT NULL,
             PRIMARY KEY (message_id, skill_slug),
             FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
         );
         CREATE INDEX idx_message_skill_selections_message
         ON message_skill_selections(message_id, sort_order);
         DELETE FROM _schema_version WHERE version = 11;",
    )
    .unwrap();
}

fn run_migrations_to_v11(conn: &Connection) {
    run_migrations(conn).unwrap();
    revert_v15(conn);
    revert_v14(conn);
    revert_v13(conn);
    conn.execute_batch(
        "DROP TABLE skill_approvals;
         DROP TABLE skill_findings;
         DROP TABLE skill_scan_runs;
         DROP TABLE skill_artifacts;
         DROP INDEX idx_skill_sources_current_scan;
         ALTER TABLE skill_sources DROP COLUMN current_scan_id;
         DELETE FROM _schema_version WHERE version = 12;",
    )
    .unwrap();
}

fn run_migrations_to_v12(conn: &Connection) {
    run_migrations(conn).unwrap();
    revert_v15(conn);
    revert_v14(conn);
    revert_v13(conn);
}

fn run_migrations_to_v14(conn: &Connection) {
    run_migrations(conn).unwrap();
    revert_v15(conn);
}

fn revert_v15(conn: &Connection) {
    conn.execute_batch(
        "DROP INDEX IF EXISTS idx_usage_import_source;
         DROP INDEX IF EXISTS idx_usage_session;
         DROP INDEX IF EXISTS idx_usage_operation;
         DROP INDEX IF EXISTS idx_usage_profile_model_date;
         DROP INDEX IF EXISTS idx_usage_profile_date;
         DROP TABLE IF EXISTS llm_usage_events;
         DROP TABLE IF EXISTS user_profiles;
         DELETE FROM _schema_version WHERE version = 15;",
    )
    .unwrap();
}

fn revert_v14(conn: &Connection) {
    conn.execute_batch(
        "DROP INDEX IF EXISTS idx_artifacts_sha256;
         DROP INDEX IF EXISTS idx_artifacts_origin_message;
         DROP INDEX IF EXISTS idx_artifacts_session_retention;
         DROP TABLE IF EXISTS artifacts;
         DROP INDEX IF EXISTS idx_message_blocks_message_position;
         DROP TABLE IF EXISTS message_blocks;
         DELETE FROM _schema_version WHERE version = 14;",
    )
    .unwrap();
}

fn revert_v13(conn: &Connection) {
    conn.execute_batch(
        "DROP TABLE skill_security_migration_items;
         DROP TABLE skill_security_migration;
         ALTER TABLE skill_sources ADD COLUMN risk_json TEXT NOT NULL DEFAULT '{}';
         CREATE TABLE skills (
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
         INSERT INTO skills (
            slug, name, description, version, source_kind, source_ref,
            source_url, checksum, installed_path, enabled, health,
            risk_json, installed_at, updated_at
         )
         SELECT slug, name, description, version, source_kind, source_ref,
                source_url, artifact_hash, installed_path, user_enabled, health,
                '{}', installed_at, updated_at
         FROM skill_sources WHERE is_managed = 1;
         CREATE INDEX idx_skills_enabled
         ON skills(enabled, health, updated_at DESC);
         DELETE FROM _schema_version WHERE version = 13;",
    )
    .unwrap();
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
