use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use misaka_x_lib::services::skills::security::scanner::scan_directory;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("skills-security")
        .join(name)
}

#[test]
fn benign_corpus_has_no_findings() {
    let report = scan_directory(&fixture("benign"), "scan-benign", &AtomicBool::new(false))
        .expect("benign fixture should scan");
    assert_eq!(report.decision.state, "passed");
    assert!(report.findings.is_empty());
}

#[test]
fn three_platform_command_and_persistence_fixtures_are_blocked() {
    for name in [
        "windows-persistence",
        "linux-download-exec",
        "macos-persistence",
    ] {
        let report = scan_directory(&fixture(name), name, &AtomicBool::new(false))
            .unwrap_or_else(|error| panic!("{name} should scan: {error}"));
        assert_eq!(report.decision.state, "blocked", "fixture {name}");
        assert!(
            report
                .findings
                .iter()
                .any(|finding| matches!(finding.severity.as_str(), "high" | "critical")),
            "fixture {name} must contain a blocking finding"
        );
    }
}

#[test]
fn prompt_injection_and_credential_access_fixtures_are_blocked() {
    for name in ["prompt-injection", "credential-access", "exfiltration"] {
        let report = scan_directory(&fixture(name), name, &AtomicBool::new(false)).unwrap();
        assert_eq!(report.decision.state, "blocked", "fixture {name}");
    }
}

#[test]
fn obfuscation_fixture_requires_human_review() {
    let report = scan_directory(
        &fixture("obfuscation"),
        "obfuscation",
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.decision.state, "review_required");
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.rule_id == "CONTENT-OBFUSCATION"));
}
