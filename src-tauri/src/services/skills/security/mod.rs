pub mod analyzers;
pub mod archive_validator;
pub mod policy;
pub mod quarantine;
pub mod scanner;
pub mod watcher;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use dashmap::DashMap;
use rusqlite::Connection;

use crate::db::repository::{SkillSecurityRepo, SkillSourceRepo};

use super::installer;
use super::types::{
    ArchiveInspection, InstallSource, SkillApprovalRecord, SkillInstallResult, SkillRecord,
    SkillScanOperation,
};

const MAX_CONCURRENT_SCANS: usize = 2;
const QUEUE_TIMEOUT: Duration = Duration::from_secs(15);

/// Unforgeable publication capability. Only this module can construct it
/// after a persisted allow decision or an audited human approval.
pub(crate) struct ApprovedArtifactId {
    scan_id: String,
    artifact_hash: String,
    extracted_path: PathBuf,
    inspection: ArchiveInspection,
}

impl ApprovedArtifactId {
    fn new(staged: quarantine::QuarantinedArchive, artifact_hash: &str) -> Result<Self> {
        if staged.inspection.checksum != artifact_hash {
            bail!("Approved artifact hash does not match quarantine inspection")
        }
        Ok(Self {
            scan_id: staged.scan_id,
            artifact_hash: artifact_hash.to_string(),
            extracted_path: staged.extracted_path,
            inspection: staged.inspection,
        })
    }

    pub(crate) fn into_parts(self) -> (String, String, PathBuf, ArchiveInspection) {
        (
            self.scan_id,
            self.artifact_hash,
            self.extracted_path,
            self.inspection,
        )
    }
}

#[derive(Default)]
pub struct ScanCoordinator {
    active: Mutex<usize>,
    available: Condvar,
    cancellation: DashMap<String, Arc<AtomicBool>>,
}

struct ScanSlot<'a> {
    coordinator: &'a ScanCoordinator,
    scan_id: String,
    pub cancelled: Arc<AtomicBool>,
}

impl Drop for ScanSlot<'_> {
    fn drop(&mut self) {
        if let Ok(mut active) = self.coordinator.active.lock() {
            *active = active.saturating_sub(1);
            self.coordinator.available.notify_one();
        }
        self.coordinator.cancellation.remove(&self.scan_id);
    }
}

impl ScanCoordinator {
    fn acquire(&self, scan_id: &str) -> Result<ScanSlot<'_>> {
        let cancelled = Arc::new(AtomicBool::new(false));
        self.cancellation
            .insert(scan_id.to_string(), Arc::clone(&cancelled));
        let started = Instant::now();
        let mut active = self
            .active
            .lock()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        while *active >= MAX_CONCURRENT_SCANS {
            if cancelled.load(Ordering::Relaxed) {
                self.cancellation.remove(scan_id);
                bail!("SCAN_CANCELLED: scan was cancelled while queued")
            }
            let remaining = QUEUE_TIMEOUT.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                self.cancellation.remove(scan_id);
                bail!("SCAN_QUEUE_TIMEOUT: scan did not start within its queue budget")
            }
            let (next, _) = self
                .available
                .wait_timeout(active, remaining.min(Duration::from_millis(250)))
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            active = next;
        }
        *active += 1;
        drop(active);
        Ok(ScanSlot {
            coordinator: self,
            scan_id: scan_id.to_string(),
            cancelled,
        })
    }

    pub fn cancel(&self, scan_id: &str) -> bool {
        self.cancellation
            .get(scan_id)
            .map(|flag| {
                flag.store(true, Ordering::Relaxed);
                self.available.notify_all();
                true
            })
            .unwrap_or(false)
    }
}

pub fn coordinator() -> &'static ScanCoordinator {
    static COORDINATOR: OnceLock<ScanCoordinator> = OnceLock::new();
    COORDINATOR.get_or_init(ScanCoordinator::default)
}

pub fn recover_startup(conn: &Connection) -> Result<usize> {
    let recovered = SkillSecurityRepo::recover_interrupted(conn)?;
    let _ = SkillSecurityRepo::cleanup_history(conn, 90)?;
    let _ = quarantine::cleanup_expired()?;
    Ok(recovered)
}

pub fn scan_and_install_archive(
    conn: &Connection,
    archive_path: &Path,
    source: InstallSource,
    mut on_progress: impl FnMut(&str, u8),
) -> Result<SkillScanOperation> {
    let scan_id = uuid::Uuid::new_v4().to_string();
    let correlation_id = uuid::Uuid::new_v4().to_string();
    on_progress(&scan_id, 0);
    let mut staged = quarantine::stage_archive(archive_path, &scan_id)?;
    let artifact_hash = super::registry::artifact_hash(&staged.extracted_path)?;
    staged.inspection.checksum = artifact_hash.clone();
    fs::write(
        staged.root.join("inspection.json"),
        serde_json::to_vec(&staged.inspection)?,
    )?;
    let source_json = serde_json::to_string(&source)?;
    SkillSecurityRepo::upsert_artifact(
        conn,
        &artifact_hash,
        staged.inspection.compressed_size_bytes,
        &source_json,
        Some(&staged.root.display().to_string()),
        "quarantined",
    )?;
    SkillSecurityRepo::create_scan(
        conn,
        &scan_id,
        &artifact_hash,
        &serde_json::json!({ "builtin": scanner::ENGINE_VERSION }).to_string(),
        policy::POLICY_VERSION,
        &correlation_id,
    )?;
    let slot = match coordinator().acquire(&scan_id) {
        Ok(slot) => slot,
        Err(error) => {
            if error.to_string().contains("SCAN_CANCELLED") {
                SkillSecurityRepo::cancel_scan(conn, &scan_id)?;
            } else {
                SkillSecurityRepo::fail_scan(
                    conn,
                    &scan_id,
                    scan_error_code(&error),
                    &error.to_string(),
                )?;
            }
            on_progress(&scan_id, 100);
            return Err(error);
        }
    };
    SkillSecurityRepo::begin_scan(conn, &scan_id)?;
    SkillSecurityRepo::set_progress(conn, &scan_id, 20)?;
    on_progress(&scan_id, 20);
    let report = match scanner::scan_directory(&staged.extracted_path, &scan_id, &slot.cancelled) {
        Ok(report) => report,
        Err(error) => {
            if error.to_string().contains("SCAN_CANCELLED") {
                SkillSecurityRepo::cancel_scan(conn, &scan_id)?;
            } else {
                SkillSecurityRepo::fail_scan(
                    conn,
                    &scan_id,
                    scan_error_code(&error),
                    &error.to_string(),
                )?;
            }
            on_progress(&scan_id, 100);
            return Err(error);
        }
    };
    SkillSecurityRepo::set_progress(conn, &scan_id, 85)?;
    on_progress(&scan_id, 85);
    SkillSecurityRepo::complete_scan(
        conn,
        &scan_id,
        report.decision.state,
        report.decision.decision,
        report.decision.max_severity,
        &report.findings,
    )?;
    let artifact_state = match report.decision.decision {
        "allow" => "approved",
        "review" => "quarantined",
        _ => "blocked",
    };
    SkillSecurityRepo::set_artifact_state(conn, &artifact_hash, artifact_state, None)?;
    let installed = if report.decision.decision == "allow" {
        let approved = ApprovedArtifactId::new(staged, &artifact_hash)?;
        let result = installer::install_approved_artifact(conn, approved, source, false)?;
        SkillSecurityRepo::attach_scan_to_source(
            conn,
            &result.skill.skill_id,
            &scan_id,
            report.decision.state,
        )?;
        SkillSecurityRepo::set_artifact_state(
            conn,
            &result.skill.checksum,
            "published",
            Some(&result.skill.installed_path),
        )?;
        Some(result.skill)
    } else {
        None
    };
    on_progress(&scan_id, 100);
    Ok(SkillScanOperation {
        scan_id,
        artifact_hash,
        state: report.decision.state.to_string(),
        decision: Some(report.decision.decision.to_string()),
        installed_skill: installed,
    })
}

pub fn scan_existing_source(
    conn: &Connection,
    record: &SkillRecord,
    mut on_progress: impl FnMut(&str, u8),
) -> Result<SkillScanOperation> {
    let scan_id = uuid::Uuid::new_v4().to_string();
    let correlation_id = uuid::Uuid::new_v4().to_string();
    let root = PathBuf::from(&record.installed_path);
    let source_json = serde_json::json!({
        "kind": record.source_kind,
        "locator": record.installed_path,
        "external": record.is_external,
    })
    .to_string();
    let size = directory_size(&root)?;
    SkillSecurityRepo::upsert_artifact(
        conn,
        &record.checksum,
        size,
        &source_json,
        None,
        "approved",
    )?;
    SkillSecurityRepo::create_scan(
        conn,
        &scan_id,
        &record.checksum,
        &serde_json::json!({ "builtin": scanner::ENGINE_VERSION }).to_string(),
        policy::POLICY_VERSION,
        &correlation_id,
    )?;
    let slot = match coordinator().acquire(&scan_id) {
        Ok(slot) => slot,
        Err(error) => {
            if error.to_string().contains("SCAN_CANCELLED") {
                SkillSecurityRepo::cancel_scan(conn, &scan_id)?;
            } else {
                SkillSecurityRepo::fail_scan(
                    conn,
                    &scan_id,
                    scan_error_code(&error),
                    &error.to_string(),
                )?;
            }
            SkillSecurityRepo::attach_scan_to_source(
                conn,
                &record.skill_id,
                &scan_id,
                "scan_error",
            )?;
            on_progress(&scan_id, 100);
            return Err(error);
        }
    };
    SkillSecurityRepo::begin_scan(conn, &scan_id)?;
    on_progress(&scan_id, 10);
    let report = match scanner::scan_directory(&root, &scan_id, &slot.cancelled) {
        Ok(report) => report,
        Err(error) => {
            if error.to_string().contains("SCAN_CANCELLED") {
                SkillSecurityRepo::cancel_scan(conn, &scan_id)?;
            } else {
                SkillSecurityRepo::fail_scan(
                    conn,
                    &scan_id,
                    scan_error_code(&error),
                    &error.to_string(),
                )?;
            }
            SkillSecurityRepo::attach_scan_to_source(
                conn,
                &record.skill_id,
                &scan_id,
                "scan_error",
            )?;
            on_progress(&scan_id, 100);
            return Err(error);
        }
    };
    SkillSecurityRepo::complete_scan(
        conn,
        &scan_id,
        report.decision.state,
        report.decision.decision,
        report.decision.max_severity,
        &report.findings,
    )?;
    SkillSecurityRepo::attach_scan_to_source(
        conn,
        &record.skill_id,
        &scan_id,
        report.decision.state,
    )?;
    on_progress(&scan_id, 100);
    Ok(SkillScanOperation {
        scan_id,
        artifact_hash: record.checksum.clone(),
        state: report.decision.state.to_string(),
        decision: Some(report.decision.decision.to_string()),
        installed_skill: Some(
            SkillSourceRepo::find(conn, &record.skill_id)?.context("Skill source disappeared")?,
        ),
    })
}

pub fn approve_scan(
    conn: &Connection,
    scan_id: &str,
    actor: &str,
    reason: &str,
    expires_at: Option<&str>,
) -> Result<(SkillApprovalRecord, SkillScanOperation)> {
    let scan = SkillSecurityRepo::scan_decision(conn, scan_id)?;
    if scan.state != "review_required" || scan.decision.as_deref() != Some("review") {
        bail!("Only a review-required scan can be approved")
    }
    let correlation_id = uuid::Uuid::new_v4().to_string();
    let approval = SkillSecurityRepo::record_approval(
        conn,
        &scan.artifact_hash,
        scan_id,
        &scan.artifact_hash,
        "approve",
        actor,
        reason,
        "artifact",
        expires_at,
        &correlation_id,
    )?;
    SkillSecurityRepo::override_scan_decision(conn, scan_id, "warnings", "allow")?;
    let installed_skill =
        if let Some(skill_id) = SkillSecurityRepo::source_id_for_scan(conn, scan_id)? {
            SkillSecurityRepo::attach_scan_to_source(conn, &skill_id, scan_id, "approved")?;
            SkillSourceRepo::find(conn, &skill_id)?
        } else {
            let artifact = SkillSecurityRepo::artifact(conn, &scan.artifact_hash)?;
            let root = artifact
                .quarantine_path
                .map(PathBuf::from)
                .context("Approved artifact has no quarantine payload")?;
            let staged = quarantine::QuarantinedArchive::load(root, scan_id)?;
            let source: InstallSource = serde_json::from_str(&artifact.source_json)?;
            let approved = ApprovedArtifactId::new(staged, &scan.artifact_hash)?;
            let result = installer::install_approved_artifact(conn, approved, source, true)?;
            SkillSecurityRepo::attach_scan_to_source(
                conn,
                &result.skill.skill_id,
                scan_id,
                "approved",
            )?;
            SkillSecurityRepo::set_artifact_state(
                conn,
                &scan.artifact_hash,
                "published",
                Some(&result.skill.installed_path),
            )?;
            Some(result.skill)
        };
    Ok((
        approval,
        SkillScanOperation {
            scan_id: scan_id.to_string(),
            artifact_hash: scan.artifact_hash,
            state: "warnings".to_string(),
            decision: Some("allow".to_string()),
            installed_skill,
        },
    ))
}

pub fn reject_scan(
    conn: &Connection,
    scan_id: &str,
    actor: &str,
    reason: &str,
) -> Result<SkillApprovalRecord> {
    let scan = SkillSecurityRepo::scan_decision(conn, scan_id)?;
    if scan.state != "review_required" {
        bail!("Only a review-required scan can be rejected")
    }
    let correlation_id = uuid::Uuid::new_v4().to_string();
    let approval = SkillSecurityRepo::record_approval(
        conn,
        &scan.artifact_hash,
        scan_id,
        &scan.artifact_hash,
        "reject",
        actor,
        reason,
        "artifact",
        None,
        &correlation_id,
    )?;
    SkillSecurityRepo::override_scan_decision(conn, scan_id, "blocked", "block")?;
    SkillSecurityRepo::set_artifact_state(conn, &scan.artifact_hash, "blocked", None)?;
    if let Some(skill_id) = SkillSecurityRepo::source_id_for_scan(conn, scan_id)? {
        SkillSecurityRepo::attach_scan_to_source(conn, &skill_id, scan_id, "blocked")?;
    }
    Ok(approval)
}

pub fn revoke_approval(conn: &Connection, approval_id: &str, skill_id: &str) -> Result<u64> {
    SkillSecurityRepo::revoke_approval(conn, approval_id)?;
    SkillSecurityRepo::mark_source_stale(conn, skill_id, "approval_revoked")
}

pub fn ensure_scan_allows(conn: &Connection, record: &SkillRecord) -> Result<()> {
    let decision = SkillSecurityRepo::decision_for_source(conn, &record.skill_id)?
        .context("SKILL_SCAN_REQUIRED: Skill has not been scanned")?;
    if decision.artifact_hash != record.checksum || !decision_versions_are_current(&decision) {
        let _ = SkillSecurityRepo::mark_source_stale(conn, &record.skill_id, "scan_version_stale");
        bail!("SKILL_SCAN_STALE: Skill artifact or policy changed")
    }
    if record.security_state == "approved"
        && !SkillSecurityRepo::has_active_approval(conn, &record.checksum, &decision.scan_id)?
    {
        bail!("SKILL_APPROVAL_EXPIRED: Skill approval is missing, expired, or revoked")
    }
    match decision.state.as_str() {
        "passed" | "warnings" => Ok(()),
        "review_required" => bail!("SKILL_REVIEW_REQUIRED: Skill requires review"),
        "blocked" => bail!("SKILL_POLICY_BLOCKED: Skill is blocked by policy"),
        "stale" => bail!("SKILL_SCAN_STALE: Skill scan is stale"),
        _ => bail!("SKILL_SCAN_REQUIRED: Skill scan is not eligible for activation"),
    }
}

pub fn ensure_external_scan(conn: &Connection, record: &SkillRecord) -> Result<()> {
    let current = SkillSecurityRepo::decision_for_source(conn, &record.skill_id)?;
    if current.as_ref().is_some_and(|scan| {
        scan.artifact_hash == record.checksum && decision_versions_are_current(scan)
    }) {
        return Ok(());
    }
    scan_existing_source(conn, record, |_, _| {}).map(|_| ())
}

fn decision_versions_are_current(
    decision: &crate::db::repository::skill_security_repo::ScanDecisionRecord,
) -> bool {
    let engines = serde_json::from_str::<serde_json::Value>(&decision.engine_versions_json).ok();
    decision.policy_version == policy::POLICY_VERSION
        && engines
            .as_ref()
            .and_then(|value| value.get("builtin"))
            .and_then(serde_json::Value::as_str)
            == Some(scanner::ENGINE_VERSION)
}

/// Trust-feed integration hook. A revoked source invalidates every matching
/// artifact before another activation can be mounted.
pub fn mark_source_artifact_revoked(
    conn: &Connection,
    artifact_hash: &str,
    reason: &str,
) -> Result<usize> {
    let mut statement =
        conn.prepare("SELECT skill_id FROM skill_sources WHERE artifact_hash = ?1")?;
    let skill_ids = statement
        .query_map([artifact_hash], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(statement);
    for skill_id in &skill_ids {
        SkillSecurityRepo::mark_source_stale(conn, skill_id, reason)?;
    }
    Ok(skill_ids.len())
}

pub fn cancel_scan(conn: &Connection, scan_id: &str) -> Result<bool> {
    let signalled = coordinator().cancel(scan_id);
    let persisted = SkillSecurityRepo::cancel_scan(conn, scan_id)?;
    Ok(signalled || persisted)
}

pub fn install_archive_compat(
    conn: &Connection,
    archive_path: &Path,
    source: InstallSource,
) -> Result<SkillInstallResult> {
    let operation = scan_and_install_archive(conn, archive_path, source, |_, _| {})?;
    operation
        .installed_skill
        .map(|skill| SkillInstallResult {
            skill,
            replaced_existing: false,
        })
        .context(format!(
            "SKILL_REVIEW_REQUIRED: scan {} produced {}",
            operation.scan_id, operation.state
        ))
}

fn directory_size(root: &Path) -> Result<u64> {
    let mut total = 0u64;
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file() {
            total = total.saturating_add(entry.metadata()?.len());
        }
    }
    Ok(total)
}

fn scan_error_code(error: &anyhow::Error) -> &str {
    let message = error.to_string();
    if message.contains("SCAN_TIMEOUT") {
        "SCAN_TIMEOUT"
    } else if message.contains("SCAN_CANCELLED") {
        "SCAN_CANCELLED"
    } else if message.contains("SCAN_LIMIT") {
        "SCAN_LIMIT_EXCEEDED"
    } else {
        "SCAN_ENGINE_ERROR"
    }
}

pub fn export_scan(
    conn: &Connection,
    scan_id: &str,
    format: &str,
    destination: &Path,
) -> Result<()> {
    let scan = SkillSecurityRepo::scan_decision(conn, scan_id)?;
    let mut findings = Vec::new();
    let mut cursor = None;
    loop {
        let page = SkillSecurityRepo::list_findings(conn, scan_id, None, cursor.as_deref(), 100)?;
        findings.extend(page.items);
        let Some(next) = page.next_cursor else { break };
        cursor = Some(next);
    }
    let payload = match format {
        "json" => serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": 1,
            "scan": {
                "scan_id": scan.scan_id,
                "artifact_hash": scan.artifact_hash,
                "state": scan.state,
                "decision": scan.decision,
                "policy_version": scan.policy_version,
            },
            "findings": findings,
        }))?,
        "sarif" => serde_json::to_vec_pretty(&serde_json::json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": { "driver": { "name": "MisakaX Built-in Skill Scanner", "version": scanner::ENGINE_VERSION } },
                "results": findings.into_iter().map(|finding| serde_json::json!({
                    "ruleId": finding.rule_id,
                    "level": match finding.severity.as_str() {
                        "critical" | "high" => "error",
                        "medium" => "warning",
                        _ => "note",
                    },
                    "message": { "text": finding.detail },
                    "locations": finding.file_path.map(|path| vec![serde_json::json!({
                        "physicalLocation": {
                            "artifactLocation": { "uri": path },
                            "region": { "startLine": finding.line_start.unwrap_or(1) }
                        }
                    })]).unwrap_or_default()
                })).collect::<Vec<_>>()
            }]
        }))?,
        _ => bail!("Unsupported scan export format"),
    };
    fs::write(destination, payload).context("Cannot write scan export")?;
    Ok(())
}

pub fn privacy_defaults() -> serde_json::Value {
    serde_json::json!({
        "virus_total_hash_lookup": false,
        "virus_total_file_upload": false,
        "cloud_llm_analysis": false,
        "network_used_by_builtin_scan": false,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;
    use std::time::Duration;

    use rusqlite::Connection;

    use crate::db::migrations::run_migrations;
    use crate::db::repository::{SkillSecurityRepo, SkillSourceRepo};
    use crate::services::skills::registry::artifact_hash;
    use crate::services::skills::types::{SkillFinding, SkillRecord, SkillRiskReport};

    use super::{
        approve_scan, ensure_scan_allows, export_scan, mark_source_artifact_revoked,
        revoke_approval, ScanCoordinator,
    };

    #[test]
    fn review_approval_is_audited_disabled_and_revocable() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("SKILL.md"),
            "---\nname: review-skill\ndescription: Review\n---\n",
        )
        .unwrap();
        let hash = artifact_hash(root.path()).unwrap();
        let record = SkillRecord {
            skill_id: String::new(),
            slug: "review-skill".to_string(),
            name: "Review".to_string(),
            description: "Review".to_string(),
            version: None,
            source_kind: "managed".to_string(),
            source_ref: None,
            source_url: None,
            checksum: hash.clone(),
            installed_path: root.path().display().to_string(),
            enabled: true,
            health: "healthy".to_string(),
            is_external: false,
            effective_active: false,
            effective_rank: 400,
            conflict: false,
            disabled_reason: None,
            security_state: "unscanned".to_string(),
            risk: SkillRiskReport::default(),
            installed_at: String::new(),
            updated_at: String::new(),
        };
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&conn, &record, &record.installed_path, true).unwrap();
        SkillSecurityRepo::upsert_artifact(&conn, &hash, 1, "{}", None, "approved").unwrap();
        SkillSecurityRepo::create_scan(
            &conn,
            "review-scan",
            &hash,
            &serde_json::json!({ "builtin": super::scanner::ENGINE_VERSION }).to_string(),
            super::policy::POLICY_VERSION,
            "correlation",
        )
        .unwrap();
        SkillSecurityRepo::begin_scan(&conn, "review-scan").unwrap();
        SkillSecurityRepo::complete_scan(
            &conn,
            "review-scan",
            "review_required",
            "review",
            Some("medium"),
            &[],
        )
        .unwrap();
        SkillSecurityRepo::attach_scan_to_source(
            &conn,
            &skill_id,
            "review-scan",
            "review_required",
        )
        .unwrap();

        let (approval, operation) =
            approve_scan(&conn, "review-scan", "tester", "Reviewed fixture", None).unwrap();
        let approvals = SkillSecurityRepo::list_approvals(&conn, "review-scan").unwrap();
        assert_eq!(approvals.len(), 1);
        assert_eq!(approvals[0].reason, "Reviewed fixture");
        assert_eq!(operation.installed_skill.unwrap().enabled, false);
        let approved = SkillSourceRepo::find(&conn, &skill_id).unwrap().unwrap();
        assert_eq!(approved.security_state, "approved");
        ensure_scan_allows(&conn, &approved).unwrap();

        revoke_approval(&conn, &approval.approval_id, &skill_id).unwrap();
        let stale = SkillSourceRepo::find(&conn, &skill_id).unwrap().unwrap();
        assert_eq!(stale.security_state, "stale");
        assert!(!stale.enabled);
        assert!(
            SkillSecurityRepo::list_approvals(&conn, "review-scan").unwrap()[0]
                .revoked_at
                .is_some()
        );
    }

    #[test]
    fn source_revocation_marks_every_matching_artifact_stale() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("SKILL.md"),
            "---\nname: revoked-skill\ndescription: Revoked\n---\n",
        )
        .unwrap();
        let hash = artifact_hash(root.path()).unwrap();
        let record = SkillRecord {
            skill_id: String::new(),
            slug: "revoked-skill".to_string(),
            name: "Revoked".to_string(),
            description: "Revoked".to_string(),
            version: None,
            source_kind: "managed".to_string(),
            source_ref: None,
            source_url: None,
            checksum: hash.clone(),
            installed_path: root.path().display().to_string(),
            enabled: true,
            health: "healthy".to_string(),
            is_external: false,
            effective_active: false,
            effective_rank: 400,
            conflict: false,
            disabled_reason: None,
            security_state: "passed".to_string(),
            risk: SkillRiskReport::default(),
            installed_at: String::new(),
            updated_at: String::new(),
        };
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&conn, &record, &record.installed_path, true).unwrap();

        assert_eq!(
            mark_source_artifact_revoked(&conn, &hash, "source_revoked").unwrap(),
            1
        );
        let stale = SkillSourceRepo::find(&conn, &skill_id).unwrap().unwrap();
        assert_eq!(stale.security_state, "stale");
        assert_eq!(stale.disabled_reason.as_deref(), Some("source_revoked"));
        assert!(!stale.enabled);
    }

    #[test]
    fn json_export_includes_every_page_and_only_redacted_evidence() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        SkillSecurityRepo::upsert_artifact(&conn, "hash", 1, "{}", None, "blocked").unwrap();
        SkillSecurityRepo::create_scan(
            &conn,
            "export-scan",
            "hash",
            &serde_json::json!({ "builtin": super::scanner::ENGINE_VERSION }).to_string(),
            super::policy::POLICY_VERSION,
            "correlation",
        )
        .unwrap();
        SkillSecurityRepo::begin_scan(&conn, "export-scan").unwrap();
        let findings = (0..101)
            .map(|index| SkillFinding {
                finding_id: format!("finding-{index:03}"),
                scan_id: "export-scan".to_string(),
                engine: super::scanner::ENGINE_VERSION.to_string(),
                rule_id: "EMBEDDED-SECRET".to_string(),
                severity: "high".to_string(),
                category: "secret".to_string(),
                file_path: Some("SKILL.md".to_string()),
                line_start: Some(index + 1),
                line_end: Some(index + 1),
                title: "Potential embedded secret".to_string(),
                detail: "Secret-shaped content was detected; the value was not stored.".to_string(),
                remediation: Some("Remove and rotate the value.".to_string()),
                fingerprint: format!("fingerprint-{index:03}"),
                evidence_redacted: Some("[REDACTED]".to_string()),
            })
            .collect::<Vec<_>>();
        SkillSecurityRepo::complete_scan(
            &conn,
            "export-scan",
            "blocked",
            "block",
            Some("high"),
            &findings,
        )
        .unwrap();
        let output = tempfile::NamedTempFile::new().unwrap();
        export_scan(&conn, "export-scan", "json", output.path()).unwrap();
        let report = fs::read_to_string(output.path()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert_eq!(json["findings"].as_array().unwrap().len(), 101);
        assert_eq!(report.matches("[REDACTED]").count(), 101);
    }

    #[test]
    fn queue_enforces_concurrency_and_cancels_waiting_scan() {
        let coordinator = Arc::new(ScanCoordinator::default());
        let first = coordinator.acquire("first").unwrap();
        let second = coordinator.acquire("second").unwrap();
        let waiting_coordinator = Arc::clone(&coordinator);
        let waiting = std::thread::spawn(move || match waiting_coordinator.acquire("waiting") {
            Ok(_) => panic!("third scan must not acquire while two slots are held"),
            Err(error) => error.to_string(),
        });
        std::thread::sleep(Duration::from_millis(25));
        assert!(coordinator.cancel("waiting"));
        let error = waiting.join().unwrap();
        assert!(error.contains("SCAN_CANCELLED"));
        drop((first, second));
    }
}
