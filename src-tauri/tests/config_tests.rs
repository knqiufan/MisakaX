use misaka_x_lib::config::AppConfig;

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert_eq!(config.theme, "system");
    assert_eq!(config.sidecar_port, 9527);
}

#[test]
fn test_config_roundtrip() {
    let config = AppConfig::default();
    let yaml = serde_yaml::to_string(&config).unwrap();
    let loaded: AppConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(config.theme, loaded.theme);
    assert_eq!(config.language, loaded.language);
}
