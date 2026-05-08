#[cfg(feature = "test-private")]
mod tests {
    use misaka_x_lib::sidecar::health_check_url;

    #[test]
    fn test_health_check_url_default_port() {
        let url = health_check_url(9527);
        assert_eq!(url, "http://127.0.0.1:9527/health");
    }

    #[test]
    fn test_health_check_url_custom_port() {
        let url = health_check_url(8080);
        assert_eq!(url, "http://127.0.0.1:8080/health");
    }

    #[test]
    fn test_health_check_url_high_port() {
        let url = health_check_url(65535);
        assert_eq!(url, "http://127.0.0.1:65535/health");
    }

    #[test]
    fn test_health_check_url_low_port() {
        let url = health_check_url(80);
        assert_eq!(url, "http://127.0.0.1:80/health");
    }

    #[test]
    fn test_sidecar_manager_start_fails_on_nonexistent_dir() {
        use misaka_x_lib::sidecar::SidecarManager;
        let result = SidecarManager::start("/nonexistent/path/to/agent", 19999);
        assert!(result.is_err());
    }
}
