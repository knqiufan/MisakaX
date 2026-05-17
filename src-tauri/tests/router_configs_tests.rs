use misaka_x_lib::crypto;
use misaka_x_lib::db::models::AdvancedConfig;
use misaka_x_lib::db::repository::RouterConfigRepo;
use rusqlite::Connection;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    misaka_x_lib::db::migrations::run_migrations(&conn).unwrap();
    conn
}

fn insert_sample_config(conn: &Connection, id: &str, provider: &str) {
    let encrypted = crypto::encrypt("sk-test-key-fake-12345678").unwrap();
    RouterConfigRepo::insert(
        conn,
        id,
        &format!("Test {}", provider),
        provider,
        &encrypted,
        Some("gpt-4o"),
        None,
        None,
        true,
        None,
    )
    .unwrap();
}

// ─── RouterConfigRepo tests ──────────────────────────────────────────

#[test]
fn test_insert_and_find_by_id() {
    let conn = setup_test_db();
    let encrypted = crypto::encrypt("sk-test-key-12345678").unwrap();

    RouterConfigRepo::insert(
        &conn,
        "rc-1",
        "My OpenAI",
        "openai",
        &encrypted,
        Some("gpt-4o"),
        Some("https://api.openai.com"),
        None,
        true,
        None,
    )
    .unwrap();

    let config = RouterConfigRepo::find_by_id(&conn, "rc-1").unwrap();
    assert_eq!(config.id, "rc-1");
    assert_eq!(config.name, "My OpenAI");
    assert_eq!(config.provider, "openai");
    assert_eq!(config.model, Some("gpt-4o".to_string()));
    assert_eq!(config.base_url, Some("https://api.openai.com".to_string()));
    assert!(config.is_active);
    assert!(config.api_key_encrypted.is_some());
}

#[test]
fn test_find_by_id_not_found() {
    let conn = setup_test_db();
    let result = RouterConfigRepo::find_by_id(&conn, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_list_all_empty() {
    let conn = setup_test_db();
    let configs = RouterConfigRepo::list_all(&conn).unwrap();
    assert!(configs.is_empty());
}

#[test]
fn test_list_all_multiple() {
    let conn = setup_test_db();
    insert_sample_config(&conn, "rc-1", "openai");
    insert_sample_config(&conn, "rc-2", "anthropic");

    let configs = RouterConfigRepo::list_all(&conn).unwrap();
    assert_eq!(configs.len(), 2);
}

#[test]
fn test_list_views_masks_api_key() {
    let conn = setup_test_db();
    insert_sample_config(&conn, "rc-1", "openai");

    let views = RouterConfigRepo::list_views(&conn).unwrap();
    assert_eq!(views.len(), 1);
    let view = &views[0];
    assert_eq!(view.id, "rc-1");
    assert!(!view.api_key_masked.contains("sk-test"));
    assert!(view.api_key_masked.contains("...") || view.api_key_masked.contains("*"));
}

#[test]
fn test_delete_existing() {
    let conn = setup_test_db();
    insert_sample_config(&conn, "rc-1", "openai");

    RouterConfigRepo::delete(&conn, "rc-1").unwrap();
    let configs = RouterConfigRepo::list_all(&conn).unwrap();
    assert!(configs.is_empty());
}

#[test]
fn test_delete_nonexistent() {
    let conn = setup_test_db();
    let result = RouterConfigRepo::delete(&conn, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_verify_exists_success() {
    let conn = setup_test_db();
    insert_sample_config(&conn, "rc-1", "openai");
    assert!(RouterConfigRepo::verify_exists(&conn, "rc-1").is_ok());
}

#[test]
fn test_verify_exists_failure() {
    let conn = setup_test_db();
    let result = RouterConfigRepo::verify_exists(&conn, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_find_connection_info() {
    let conn = setup_test_db();
    insert_sample_config(&conn, "rc-1", "openai");

    let (api_key, base_url, provider, vendor) =
        RouterConfigRepo::find_connection_info(&conn, "rc-1").unwrap();

    assert!(api_key.is_some());
    assert!(base_url.is_none());
    assert_eq!(provider, "openai");
    assert_eq!(vendor, None);
}

#[test]
fn test_find_views_by_optional_id_all() {
    let conn = setup_test_db();
    insert_sample_config(&conn, "rc-1", "openai");
    insert_sample_config(&conn, "rc-2", "anthropic");

    let views = RouterConfigRepo::find_views_by_optional_id(&conn, None).unwrap();
    assert_eq!(views.len(), 2);
}

#[test]
fn test_find_views_by_optional_id_specific() {
    let conn = setup_test_db();
    insert_sample_config(&conn, "rc-1", "openai");
    insert_sample_config(&conn, "rc-2", "anthropic");

    let views = RouterConfigRepo::find_views_by_optional_id(&conn, Some("rc-1")).unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].id, "rc-1");
}

#[test]
fn test_insert_with_api_compat() {
    let conn = setup_test_db();
    let encrypted = crypto::encrypt("sk-key-123").unwrap();

    RouterConfigRepo::insert(
        &conn,
        "rc-deepseek",
        "DeepSeek",
        "deepseek",
        &encrypted,
        None,
        Some("https://api.deepseek.com"),
        None,
        true,
        Some("openai"),
    )
    .unwrap();

    let config = RouterConfigRepo::find_by_id(&conn, "rc-deepseek").unwrap();
    assert_eq!(config.provider, "deepseek");
    assert_eq!(config.api_compat, Some("openai".to_string()));
    assert_eq!(
        config.base_url,
        Some("https://api.deepseek.com".to_string())
    );
}

#[test]
fn test_insert_with_metadata_round_trips_vendor_and_advanced() {
    let conn = setup_test_db();
    let encrypted = crypto::encrypt("sk-key-123").unwrap();
    let advanced = serde_json::to_string(&AdvancedConfig {
        temperature: 0.2,
        max_tokens: Some(1024),
        proxy: None,
    })
    .unwrap();

    RouterConfigRepo::insert_with_metadata(
        &conn,
        "rc-zhipu",
        "Zhipu",
        "openai",
        Some("zhipu"),
        &encrypted,
        None,
        Some("https://open.bigmodel.cn/api/paas/v4"),
        None,
        Some(&advanced),
        true,
        Some("openai"),
    )
    .unwrap();

    let config = RouterConfigRepo::find_by_id(&conn, "rc-zhipu").unwrap();
    let view = RouterConfigRepo::find_view_by_id(&conn, "rc-zhipu").unwrap();

    assert_eq!(config.vendor, Some("zhipu".to_string()));
    assert_eq!(config.provider, "openai");
    assert_eq!(view.vendor, Some("zhipu".to_string()));
    assert_eq!(view.advanced.temperature, 0.2);
    assert_eq!(view.advanced.max_tokens, Some(1024));
}
