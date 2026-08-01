use crate::services::skills::types::SkillManifest;

use super::FindingCollector;

pub fn inspect(
    manifest: &SkillManifest,
    saw_script: bool,
    saw_network_behavior: bool,
    collector: &mut FindingCollector,
) {
    if saw_script && manifest.allowed_tools.is_none() {
        collector.add(
            "PERMISSION-SCRIPT-UNDECLARED",
            "medium",
            "permission_mismatch",
            Some("SKILL.md"),
            None,
            "Bundled scripts are not reflected in the declared tool requirements",
            "The artifact contains scripts but the manifest does not declare allowed tools.",
            "Declare the minimal required tools and explain why scripts are needed.",
            None,
        );
    }
    if saw_network_behavior
        && !manifest
            .allowed_tools
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains("network")
    {
        collector.add(
            "PERMISSION-NETWORK-UNDECLARED",
            "medium",
            "permission_mismatch",
            Some("SKILL.md"),
            None,
            "Network behavior is not declared",
            "Instructions or scripts reference network access without a matching declaration.",
            "Declare the minimal network requirement and expected destinations.",
            None,
        );
    }
}
