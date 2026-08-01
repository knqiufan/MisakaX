mod command_rule_analyzer;
mod content_inventory;
mod manifest_analyzer;
mod permission_consistency_analyzer;
mod secret_analyzer;

use std::collections::HashSet;

use sha2::{Digest, Sha256};

use crate::services::skills::types::{SkillFinding, SkillManifest};

use super::scanner::ENGINE_VERSION;

pub struct FindingCollector {
    scan_id: String,
    findings: Vec<SkillFinding>,
    fingerprints: HashSet<String>,
}

impl FindingCollector {
    pub fn new(scan_id: &str) -> Self {
        Self {
            scan_id: scan_id.to_string(),
            findings: Vec::new(),
            fingerprints: HashSet::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &mut self,
        rule_id: &str,
        severity: &str,
        category: &str,
        file_path: Option<&str>,
        line: Option<u32>,
        title: &str,
        detail: &str,
        remediation: &str,
        evidence_redacted: Option<&str>,
    ) {
        let fingerprint = fingerprint(rule_id, file_path, line, title);
        if !self.fingerprints.insert(fingerprint.clone()) {
            return;
        }
        self.findings.push(SkillFinding {
            finding_id: uuid::Uuid::new_v4().to_string(),
            scan_id: self.scan_id.clone(),
            engine: ENGINE_VERSION.to_string(),
            rule_id: rule_id.to_string(),
            severity: severity.to_string(),
            category: category.to_string(),
            file_path: file_path.map(str::to_string),
            line_start: line,
            line_end: line,
            title: title.to_string(),
            detail: detail.to_string(),
            remediation: Some(remediation.to_string()),
            fingerprint,
            evidence_redacted: evidence_redacted.map(str::to_string),
        });
    }

    pub fn into_findings(self) -> Vec<SkillFinding> {
        self.findings
    }
}

pub fn inspect_path(path: &str, collector: &mut FindingCollector) {
    content_inventory::inspect_path(path, collector);
}

pub fn inspect_file_signature(
    path: &str,
    extension: &str,
    bytes: &[u8],
    collector: &mut FindingCollector,
) {
    content_inventory::inspect_file_signature(path, extension, bytes, collector);
}

pub fn is_executable(bytes: &[u8]) -> bool {
    content_inventory::is_executable(bytes)
}

pub fn inspect_line(
    path: &str,
    line_number: u32,
    original: &str,
    lower: &str,
    collector: &mut FindingCollector,
) {
    command_rule_analyzer::inspect_line(path, line_number, lower, collector);
    secret_analyzer::inspect_line(path, line_number, original, lower, collector);
}

pub fn parse_manifest(markdown: &str) -> anyhow::Result<SkillManifest> {
    manifest_analyzer::parse(markdown)
}

pub fn inspect_permissions(
    manifest: &SkillManifest,
    saw_script: bool,
    saw_network_behavior: bool,
    collector: &mut FindingCollector,
) {
    permission_consistency_analyzer::inspect(manifest, saw_script, saw_network_behavior, collector);
}

fn fingerprint(rule_id: &str, path: Option<&str>, line: Option<u32>, title: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ENGINE_VERSION.as_bytes());
    hasher.update([0]);
    hasher.update(rule_id.as_bytes());
    hasher.update([0]);
    hasher.update(path.unwrap_or_default().as_bytes());
    hasher.update([0]);
    hasher.update(line.unwrap_or_default().to_le_bytes());
    hasher.update([0]);
    hasher.update(title.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(super) fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
