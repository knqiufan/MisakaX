use misaka_x_lib::config::AppConfig;
use misaka_x_lib::db::repository::SettingsRepo;
use rusqlite::Connection;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    misaka_x_lib::db::migrations::run_migrations(&conn).unwrap();
    conn
}

// ─── SettingsRepo tests ──────────────────────────────────────────────

#[test]
fn test_settings_get_nonexistent_key() {
    let conn = setup_test_db();
    let result = SettingsRepo::get(&conn, "nonexistent_key").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_settings_set_and_get() {
    let conn = setup_test_db();
    SettingsRepo::set(&conn, "theme", "dark").unwrap();
    let value = SettingsRepo::get(&conn, "theme").unwrap();
    assert_eq!(value, Some("dark".to_string()));
}

#[test]
fn test_settings_set_overwrites_existing() {
    let conn = setup_test_db();
    SettingsRepo::set(&conn, "lang", "en").unwrap();
    SettingsRepo::set(&conn, "lang", "zh_CN").unwrap();
    let value = SettingsRepo::get(&conn, "lang").unwrap();
    assert_eq!(value, Some("zh_CN".to_string()));
}

#[test]
fn test_settings_get_all_empty() {
    let conn = setup_test_db();
    let all = SettingsRepo::get_all(&conn).unwrap();
    assert!(all.is_empty());
}

#[test]
fn test_settings_get_all_multiple() {
    let conn = setup_test_db();
    SettingsRepo::set(&conn, "key1", "val1").unwrap();
    SettingsRepo::set(&conn, "key2", "val2").unwrap();
    SettingsRepo::set(&conn, "key3", "val3").unwrap();

    let all = SettingsRepo::get_all(&conn).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all.get("key1"), Some(&"val1".to_string()));
    assert_eq!(all.get("key2"), Some(&"val2".to_string()));
    assert_eq!(all.get("key3"), Some(&"val3".to_string()));
}

// ─── AppConfig tests ──────────────────────────────────────────────────

#[test]
fn test_app_config_serialize_deserialize() {
    let config = AppConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.theme, config.theme);
    assert_eq!(deserialized.language, config.language);
    assert_eq!(deserialized.sidecar_port, config.sidecar_port);
}

#[test]
fn test_app_config_default_values() {
    let config = AppConfig::default();
    assert_eq!(config.language, "en");
    assert_eq!(config.theme, "system");
    assert_eq!(config.accent_color, "#6366f1");
    assert!(!config.reduced_transparency);
    assert_eq!(config.ui_font_size, 14);
    assert_eq!(config.log_level, "info");
    assert_eq!(config.sidecar_port, 9527);
    assert!(config.auto_start_sidecar);
}

#[test]
fn test_app_config_yaml_roundtrip() {
    let config = AppConfig {
        language: "zh_CN".to_string(),
        theme: "dark".to_string(),
        accent_color: "#F43F5E".to_string(),
        reduced_transparency: true,
        ui_font_size: 16,
        default_model: "gpt-4o".to_string(),
        log_level: "debug".to_string(),
        sidecar_port: 8888,
        auto_start_sidecar: false,
    };

    let yaml = serde_yaml::to_string(&config).unwrap();
    let loaded: AppConfig = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(loaded.language, "zh_CN");
    assert_eq!(loaded.theme, "dark");
    assert_eq!(loaded.accent_color, "#F43F5E");
    assert!(loaded.reduced_transparency);
    assert_eq!(loaded.ui_font_size, 16);
    assert_eq!(loaded.default_model, "gpt-4o");
    assert_eq!(loaded.log_level, "debug");
    assert_eq!(loaded.sidecar_port, 8888);
    assert!(!loaded.auto_start_sidecar);
}

#[test]
fn test_app_config_missing_fields_use_defaults() {
    let yaml = "language: en\ntheme: light\n";
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.ui_font_size, 14);
    assert!(!config.reduced_transparency);
    assert!(config.auto_start_sidecar);
}
