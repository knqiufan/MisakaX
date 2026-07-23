use misaka_x_lib::config::{AppConfig, CloseBehavior};

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert_eq!(config.theme, "system");
    assert_eq!(config.sidecar_port, 9527);
    assert!(config.use_sidecar);
    assert_eq!(config.mcp_bridge_port, 9528);
    assert_eq!(config.close_behavior, CloseBehavior::Ask);
}

#[test]
fn test_config_roundtrip() {
    let config = AppConfig::default();
    let yaml = serde_yaml::to_string(&config).unwrap();
    let loaded: AppConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(config.theme, loaded.theme);
    assert_eq!(config.language, loaded.language);
    assert_eq!(config.use_sidecar, loaded.use_sidecar);
    assert_eq!(config.mcp_bridge_port, loaded.mcp_bridge_port);
    assert_eq!(config.close_behavior, loaded.close_behavior);
}

#[test]
fn test_use_sidecar_missing_field_defaults_true() {
    let yaml = r#"
language: en
theme: light
sidecar_port: 9527
auto_start_sidecar: true
"#;
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.use_sidecar);
}

#[test]
fn test_use_sidecar_can_be_disabled() {
    let yaml = r#"
language: en
theme: system
use_sidecar: false
"#;
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(!config.use_sidecar);
}

#[test]
fn test_close_behavior_missing_field_defaults_to_ask() {
    let yaml = "language: en\ntheme: system\n";
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.close_behavior, CloseBehavior::Ask);
}

#[test]
fn test_close_behavior_serializes_as_stable_snake_case() {
    let mut config = AppConfig::default();
    config.close_behavior = CloseBehavior::MinimizeToTray;
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("close_behavior: minimize_to_tray"));
}
