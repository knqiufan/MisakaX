use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn read(relative: &str) -> String {
    fs::read_to_string(manifest_path(relative)).unwrap()
}

fn json(relative: &str) -> Value {
    serde_json::from_str(&read(relative)).unwrap()
}

fn string_set(values: &Value) -> BTreeSet<String> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect()
}

fn quoted_list(source: &str, marker: &str) -> BTreeSet<String> {
    let mut inside = false;
    let mut values = BTreeSet::new();
    for line in source.lines() {
        if !inside {
            inside = line.contains(marker);
            continue;
        }
        if line.trim_start().starts_with("];") || line.trim_start().starts_with(']') {
            break;
        }
        let trimmed = line.trim().trim_end_matches(',');
        if let Some(value) = trimmed.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
            values.insert(value.to_owned());
        }
    }
    values
}

fn handler_commands(source: &str) -> BTreeSet<String> {
    let mut inside = false;
    let mut commands = BTreeSet::new();
    for line in source.lines() {
        if !inside {
            inside = line.contains("invoke_handler(tauri::generate_handler!");
            continue;
        }
        if line.trim_start().starts_with("])") {
            break;
        }
        if let Some(command) = line
            .trim()
            .trim_end_matches(',')
            .rsplit("::")
            .next()
            .filter(|_| line.contains("commands::"))
        {
            commands.insert(command.to_owned());
        }
    }
    commands
}

#[test]
fn w5_webview_capability_is_local_main_window_only_and_minimal() {
    let capability = json("capabilities/default.json");
    assert_eq!(capability["windows"], serde_json::json!(["main"]));
    assert!(capability.get("remote").is_none());

    let permissions = string_set(&capability["permissions"]);
    let expected = BTreeSet::from([
        "core:event:allow-listen".to_owned(),
        "core:event:allow-unlisten".to_owned(),
        "shell:allow-open".to_owned(),
        "dialog:allow-open".to_owned(),
        "dialog:allow-save".to_owned(),
        "clipboard-manager:allow-write-text".to_owned(),
        "clipboard-manager:allow-read-text".to_owned(),
        "main-commands".to_owned(),
        "terminal-runtime".to_owned(),
    ]);
    assert_eq!(permissions, expected);

    for forbidden in [
        "shell:allow-execute",
        "shell:allow-spawn",
        "shell:allow-stdin-write",
        "shell:allow-kill",
        "fs:default",
        "fs:allow-read",
        "fs:allow-write",
        "fs:allow-remove",
        "http:default",
        "http:allow-fetch",
        "notification:default",
        "core:default",
    ] {
        assert!(
            !permissions.contains(forbidden),
            "forbidden permission {forbidden}"
        );
    }

    let config = json("tauri.conf.json");
    assert_eq!(
        config.pointer("/app/security/capabilities"),
        Some(&serde_json::json!(["default"]))
    );
}

#[test]
fn w5_every_registered_custom_command_is_manifested_and_authorized() {
    let registered = handler_commands(&read("src/lib.rs"));
    let manifested = quoted_list(&read("build.rs"), "const COMMANDS:");
    assert_eq!(
        manifested, registered,
        "AppManifest and invoke_handler drifted"
    );

    let main = quoted_list(&read("permissions/main.toml"), "commands.allow");
    let terminal = quoted_list(&read("permissions/terminal.toml"), "commands.allow");
    let expected_terminal = BTreeSet::from([
        "terminal_spawn".to_owned(),
        "terminal_write".to_owned(),
        "terminal_resize".to_owned(),
        "terminal_kill".to_owned(),
        "terminal_get_state".to_owned(),
    ]);
    assert_eq!(terminal, expected_terminal);
    assert!(main.is_disjoint(&terminal));
    assert_eq!(
        main.union(&terminal).cloned().collect::<BTreeSet<_>>(),
        manifested
    );
}

#[test]
fn w5_production_csp_is_strict_and_dev_exceptions_stay_local() {
    let config = json("tauri.conf.json");
    let security = config.pointer("/app/security").unwrap();
    let csp = security["csp"].as_object().unwrap();
    let dev_csp = security["devCsp"].as_object().unwrap();

    assert_eq!(csp["script-src"], "'self'");
    assert_eq!(csp["object-src"], "'none'");
    assert_eq!(csp["frame-src"], "'none'");
    assert_eq!(csp["base-uri"], "'none'");
    assert_eq!(csp["form-action"], "'none'");
    assert_eq!(csp["connect-src"], "'self' ipc: http://ipc.localhost");
    assert_eq!(security["dangerousDisableAssetCspModification"], false);

    let production = csp
        .values()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .unwrap()
        .join(" ");
    for forbidden in ["unsafe-eval", "https://", "localhost:1420", "ws://", " * "] {
        assert!(
            !production.contains(forbidden),
            "production CSP contains {forbidden}"
        );
    }

    assert!(dev_csp["connect-src"]
        .as_str()
        .unwrap()
        .contains("ws://localhost:1420"));
    assert!(dev_csp["default-src"]
        .as_str()
        .unwrap()
        .contains("http://localhost:1420"));
}

#[test]
fn w5_unused_webview_plugins_and_terminal_dom_sinks_are_absent() {
    let cargo = read("Cargo.toml");
    let lib = read("src/lib.rs");
    let package = fs::read_to_string(manifest_path("../package.json")).unwrap();
    for plugin in ["plugin-fs", "plugin-http", "plugin-notification"] {
        assert!(!cargo.contains(plugin));
        assert!(!package.contains(plugin));
    }
    for init in [
        "tauri_plugin_fs::init",
        "tauri_plugin_http::init",
        "tauri_plugin_notification::init",
    ] {
        assert!(!lib.contains(init));
    }

    let terminal_panel = fs::read_to_string(manifest_path(
        "../src/components/chat/workspace/TerminalPanel.tsx",
    ))
    .unwrap();
    for sink in [
        "innerHTML",
        "dangerouslySetInnerHTML",
        "WebLinksAddon",
        "registerLinkProvider",
        "window.open",
    ] {
        assert!(
            !terminal_panel.contains(sink),
            "TerminalPanel contains {sink}"
        );
    }

    assert!(Path::new(&manifest_path("permissions/main.toml")).is_file());
    assert!(Path::new(&manifest_path("permissions/terminal.toml")).is_file());
}

#[test]
fn w5_workspace_and_terminal_events_use_tauri_safe_names() {
    let backend = read("src/lib.rs");
    let workspace = fs::read_to_string(manifest_path("../src/lib/ipc/workspace.ts")).unwrap();
    let terminal = fs::read_to_string(manifest_path("../src/lib/ipc/terminal.ts")).unwrap();

    for event in [
        "workspace:context:changed",
        "terminal:output",
        "terminal:exited",
    ] {
        assert!(backend.contains(event), "backend event missing {event}");
        assert!(
            workspace.contains(event) || terminal.contains(event),
            "frontend event missing {event}"
        );
        assert!(event
            .chars()
            .all(|character| character.is_alphanumeric()
                || matches!(character, '-' | '/' | ':' | '_')));
    }

    for invalid in [
        "workspace.context.changed",
        "terminal.output",
        "terminal.exited",
    ] {
        assert!(!backend.contains(invalid));
        assert!(!workspace.contains(invalid));
        assert!(!terminal.contains(invalid));
    }
}

#[test]
fn isolation_s0_has_no_host_process_or_webview_execution_shortcut() {
    let sandbox_root = manifest_path("src/services/sandbox");
    let production_files = [
        "mod.rs",
        "types.rs",
        "provider.rs",
        "snapshot.rs",
        "broker.rs",
    ];
    let source = production_files
        .iter()
        .map(|name| fs::read_to_string(sandbox_root.join(name)).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    for forbidden in [
        "std::process::Command",
        "tokio::process::Command",
        "portable_pty",
        "tauri_plugin_shell",
        "Command::new(",
        "powershell",
        "cmd.exe",
        "sandbox-exec",
    ] {
        assert!(
            !source.contains(forbidden),
            "S0 production isolation contract contains host execution shortcut {forbidden}"
        );
    }

    let module = fs::read_to_string(sandbox_root.join("mod.rs")).unwrap();
    assert!(module
        .lines()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|lines| lines == ["#[cfg(test)]", "mod tests;"]));
    assert!(!source.contains("FakeProvider"));

    let capability = json("capabilities/default.json");
    let permissions = string_set(&capability["permissions"]);
    assert!(!permissions.contains("shell:allow-execute"));
    assert!(!permissions.contains("shell:allow-spawn"));
}
