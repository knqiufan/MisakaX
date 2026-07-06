#[cfg(feature = "test-private")]
mod tests {
    use misaka_x_lib::sidecar::{
        health_check_url, is_current_watchdog_generation, should_attempt_runtime_restart,
        SidecarStatus, SidecarStatusEvent,
    };
    use serde_json::json;

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
    fn test_sidecar_status_serializes_to_snake_case() {
        let cases = [
            (SidecarStatus::Stopped, json!("stopped")),
            (SidecarStatus::Starting, json!("starting")),
            (SidecarStatus::Ready, json!("ready")),
            (SidecarStatus::Error, json!("error")),
            (SidecarStatus::Restarting, json!("restarting")),
        ];

        for (status, expected) in cases {
            assert_eq!(serde_json::to_value(status).unwrap(), expected);
        }
    }

    #[test]
    fn test_sidecar_status_event_serializes_frontend_payload() {
        let event = SidecarStatusEvent {
            status: SidecarStatus::Ready,
            message: None,
            port: 9527,
        };

        assert_eq!(
            serde_json::to_value(event).unwrap(),
            json!({
                "status": "ready",
                "message": null,
                "port": 9527
            })
        );
    }

    #[test]
    fn test_sidecar_status_event_preserves_error_message() {
        let event = SidecarStatusEvent {
            status: SidecarStatus::Error,
            message: Some("runtime restart limit exceeded".to_string()),
            port: 9527,
        };

        assert_eq!(
            serde_json::to_value(event).unwrap(),
            json!({
                "status": "error",
                "message": "runtime restart limit exceeded",
                "port": 9527
            })
        );
    }

    #[test]
    fn test_runtime_restart_policy_allows_first_three_attempts() {
        for next_count in 1..=3 {
            assert!(should_attempt_runtime_restart(next_count, 3));
        }
    }

    #[test]
    fn test_runtime_restart_policy_rejects_fourth_attempt() {
        assert!(!should_attempt_runtime_restart(4, 3));
    }

    #[test]
    fn test_watchdog_generation_policy_rejects_stale_detection() {
        assert!(is_current_watchdog_generation(2, 2));
        assert!(!is_current_watchdog_generation(1, 2));
    }
}
