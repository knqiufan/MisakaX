use std::{fs, path::PathBuf};

use serde_json::Value;

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn w0_records_the_legacy_webview_capability_baseline() {
    let value: Value = serde_json::from_str(
        &fs::read_to_string(manifest_path("capabilities/default.json")).unwrap(),
    )
    .unwrap();
    let permissions = value["permissions"].as_array().unwrap();
    let has = |permission: &str| permissions.iter().any(|value| value == permission);

    assert!(has("shell:allow-execute"));
    assert!(has("shell:allow-spawn"));
    assert!(has("shell:allow-stdin-write"));
    assert!(has("shell:allow-kill"));
    assert!(has("fs:allow-write"));
    assert!(has("http:allow-fetch"));
}

#[test]
fn w0_records_that_production_csp_is_not_yet_configured() {
    let value: Value =
        serde_json::from_str(&fs::read_to_string(manifest_path("tauri.conf.json")).unwrap())
            .unwrap();
    assert!(value["app"]["security"]["csp"].is_null());
}
