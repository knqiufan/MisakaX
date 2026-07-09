#[cfg(feature = "test-private")]
mod tests {
    use misaka_x_lib::sidecar::{
        find_sidecar_executable_in, health_check_url, is_current_watchdog_generation,
        resolve_sidecar_executable, should_attempt_runtime_restart, sidecar_executable_name,
        SidecarStatus, SidecarStatusEvent, SIDECAR_BINARY_STEM,
    };
    use serde_json::json;
    use std::path::PathBuf;

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

    #[test]
    fn test_runtime_restart_policy_rejects_zero_max() {
        // With a zero budget, even the first attempt must be refused.
        assert!(!should_attempt_runtime_restart(1, 0));
    }

    #[test]
    fn test_runtime_restart_policy_boundary_matches_max() {
        assert!(should_attempt_runtime_restart(5, 5));
        assert!(!should_attempt_runtime_restart(6, 5));
    }

    #[test]
    fn test_sidecar_executable_name_has_platform_suffix() {
        let name = sidecar_executable_name();
        assert!(name.starts_with(SIDECAR_BINARY_STEM));
        if cfg!(windows) {
            assert_eq!(name, "misaka-agent.exe");
        } else {
            assert_eq!(name, "misaka-agent");
        }
    }

    #[test]
    fn test_find_sidecar_executable_returns_none_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let dirs = vec![dir.path().to_path_buf()];
        assert!(find_sidecar_executable_in(&dirs, "misaka-agent").is_none());
    }

    #[test]
    fn test_find_sidecar_executable_finds_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("misaka-agent");
        std::fs::write(&exe, b"stub").unwrap();

        let dirs = vec![dir.path().to_path_buf()];
        let found = find_sidecar_executable_in(&dirs, "misaka-agent");
        assert_eq!(found, Some(exe));
    }

    #[test]
    fn test_find_sidecar_executable_skips_directory_entries() {
        // A directory named like the binary must not be treated as a match.
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("misaka-agent")).unwrap();

        let dirs = vec![dir.path().to_path_buf()];
        assert!(find_sidecar_executable_in(&dirs, "misaka-agent").is_none());
    }

    #[test]
    fn test_find_sidecar_executable_respects_probe_order() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();

        // Only the second directory has the binary; still resolved.
        let exe = second.path().join("misaka-agent");
        std::fs::write(&exe, b"stub").unwrap();

        let dirs = vec![first.path().to_path_buf(), second.path().to_path_buf()];
        assert_eq!(find_sidecar_executable_in(&dirs, "misaka-agent"), Some(exe));
    }

    #[test]
    fn test_resolve_sidecar_executable_none_for_source_only_dir() {
        // Agent dir without a `dist/` binary → dev fallback (None), unless the
        // test runner happens to sit next to a packaged binary.
        let dir = tempfile::tempdir().unwrap();
        let resolved = resolve_sidecar_executable(dir.path());

        if let Some(path) = resolved {
            // Only acceptable if it came from the executable-sibling probe.
            assert!(!path.starts_with(dir.path()));
        }
    }

    #[test]
    fn test_resolve_sidecar_executable_finds_dist_binary() {
        let dir = tempfile::tempdir().unwrap();
        let dist = dir.path().join("dist");
        std::fs::create_dir(&dist).unwrap();
        let exe = dist.join(sidecar_executable_name());
        std::fs::write(&exe, b"stub").unwrap();

        let resolved = resolve_sidecar_executable(dir.path());
        assert_eq!(resolved, Some(exe));
    }

    #[test]
    fn test_resolve_sidecar_executable_type_is_pathbuf() {
        // Compile-time guard that the public signature returns Option<PathBuf>.
        let _f: fn(&std::path::Path) -> Option<PathBuf> = resolve_sidecar_executable;
    }
}
