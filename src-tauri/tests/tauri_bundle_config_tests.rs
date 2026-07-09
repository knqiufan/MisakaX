use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn bundle_icon_paths_exist() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = load_tauri_config(manifest_dir);
    let icons = config
        .pointer("/bundle/icon")
        .and_then(Value::as_array)
        .expect("tauri.conf.json should declare bundle.icon as an array");

    for icon in icons {
        let icon_path = icon
            .as_str()
            .expect("bundle.icon entries should be relative paths");
        let absolute_path = manifest_dir.join(icon_path);

        assert!(
            absolute_path.is_file(),
            "bundle.icon entry `{}` does not exist at {}",
            icon_path,
            absolute_path.display()
        );
    }
}

fn load_tauri_config(manifest_dir: &Path) -> Value {
    let config_path = manifest_dir.join("tauri.conf.json");
    let config = fs::read_to_string(&config_path).unwrap_or_else(|error| {
        panic!(
            "failed to read Tauri config at {}: {}",
            config_path.display(),
            error
        )
    });

    serde_json::from_str(&config).expect("tauri.conf.json should be valid JSON")
}
