use misaka_x_lib::config::AppConfig;

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert_eq!(config.theme, "system");
    assert_eq!(config.sidecar_port, 9527);
    assert!(config.use_sidecar);
    assert_eq!(config.mcp_bridge_port, 9528);
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
