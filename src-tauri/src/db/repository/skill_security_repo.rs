use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::services::skills::types::{
    SkillApprovalRecord, SkillFinding, SkillFindingPage, SkillScanSummary,
};

use super::SkillSourceRepo;

pub struct SkillSecurityRepo;

#[derive(Debug, Clone)]
pub struct ScanDecisionRecord {
    pub scan_id: String,
    pub artifact_hash: String,
    pub state: String,
    pub decision: Option<String>,
    pub engine_versions_json: String,
    pub policy_version: String,
}

#[derive(Debug, Clone)]
pub struct ArtifactRecord {
    pub artifact_hash: String,
    pub size_bytes: u64,
    pub source_json: String,
    pub quarantine_path: Option<String>,
    pub installed_path: Option<String>,
    pub state: String,
}

impl SkillSecurityRepo {
    pub fn upsert_artifact(
        conn: &Connection,
        artifact_hash: &str,
        size_bytes: u64,
        source_json: &str,
        quarantine_path: Option<&str>,
        state: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO skill_artifacts (
                artifact_hash, size_bytes, source_json, quarantine_path, state
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(artifact_hash) DO UPDATE SET
                size_bytes = excluded.size_bytes,
                source_json = excluded.source_json,
                quarantine_path = COALESCE(excluded.quarantine_path, skill_artifacts.quarantine_path),
                state = excluded.state,
                updated_at = CURRENT_TIMESTAMP",
            params![artifact_hash, size_bytes, source_json, quarantine_path, state],
        )?;
        Ok(())
    }

    pub fn create_scan(
        conn: &Connection,
        scan_id: &str,
        artifact_hash: &str,
        engine_versions_json: &str,
        policy_version: &str,
        correlation_id: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO skill_scan_runs (
                scan_id, artifact_hash, state, engine_versions_json,
                policy_version, correlation_id
             ) VALUES (?1, ?2, 'queued', ?3, ?4, ?5)",
            params![
                scan_id,
                artifact_hash,
                engine_versions_json,
                policy_version,
                correlation_id
            ],
        )?;
        Ok(())
    }

    pub fn artifact(conn: &Connection, artifact_hash: &str) -> Result<ArtifactRecord> {
        conn.query_row(
            "SELECT artifact_hash, size_bytes, source_json, quarantine_path,
                    installed_path, state
             FROM skill_artifacts WHERE artifact_hash = ?1",
            [artifact_hash],
            |row| {
                Ok(ArtifactRecord {
                    artifact_hash: row.get(0)?,
                    size_bytes: row.get(1)?,
                    source_json: row.get(2)?,
                    quarantine_path: row.get(3)?,
                    installed_path: row.get(4)?,
                    state: row.get(5)?,
                })
            },
        )
        .optional()?
        .context("Skill artifact does not exist")
    }

    pub fn set_artifact_state(
        conn: &Connection,
        artifact_hash: &str,
        state: &str,
        installed_path: Option<&str>,
    ) -> Result<()> {
        let changed = conn.execute(
            "UPDATE skill_artifacts
             SET state = ?1, installed_path = COALESCE(?2, installed_path),
                 updated_at = CURRENT_TIMESTAMP
             WHERE artifact_hash = ?3",
            params![state, installed_path, artifact_hash],
        )?;
        if changed != 1 {
            bail!("Skill artifact does not exist")
        }
        Ok(())
    }

    pub fn begin_scan(conn: &Connection, scan_id: &str) -> Result<()> {
        let changed = conn.execute(
            "UPDATE skill_scan_runs
             SET state = 'scanning', progress = 1, started_at = CURRENT_TIMESTAMP,
                 error_code = NULL, error_detail = NULL
             WHERE scan_id = ?1 AND state = 'queued'",
            [scan_id],
        )?;
        if changed != 1 {
            bail!("Scan is not queued")
        }
        Ok(())
    }

    pub fn set_progress(conn: &Connection, scan_id: &str, progress: u8) -> Result<()> {
        conn.execute(
            "UPDATE skill_scan_runs SET progress = ?1
             WHERE scan_id = ?2 AND state = 'scanning'",
            params![progress.min(99), scan_id],
        )?;
        Ok(())
    }

    pub fn complete_scan(
        conn: &Connection,
        scan_id: &str,
        state: &str,
        decision: &str,
        max_severity: Option<&str>,
        findings: &[SkillFinding],
    ) -> Result<()> {
        let transaction = conn.unchecked_transaction()?;
        transaction.execute("DELETE FROM skill_findings WHERE scan_id = ?1", [scan_id])?;
        for finding in findings {
            transaction.execute(
                "INSERT INTO skill_findings (
                    finding_id, scan_id, engine, rule_id, severity, category,
                    file_path, line_start, line_end, title, detail, remediation,
                    fingerprint, evidence_redacted
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
                 )",
                params![
                    finding.finding_id,
                    scan_id,
                    finding.engine,
                    finding.rule_id,
                    finding.severity,
                    finding.category,
                    finding.file_path,
                    finding.line_start,
                    finding.line_end,
                    finding.title,
                    finding.detail,
                    finding.remediation,
                    finding.fingerprint,
                    finding.evidence_redacted,
                ],
            )?;
        }
        let changed = transaction.execute(
            "UPDATE skill_scan_runs
             SET state = ?1, decision = ?2, max_severity = ?3, progress = 100,
                 completed_at = CURRENT_TIMESTAMP, error_code = NULL, error_detail = NULL
             WHERE scan_id = ?4 AND state = 'scanning'",
            params![state, decision, max_severity, scan_id],
        )?;
        if changed != 1 {
            bail!("Scan is not running")
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn fail_scan(
        conn: &Connection,
        scan_id: &str,
        error_code: &str,
        error_detail: &str,
    ) -> Result<()> {
        conn.execute(
            "UPDATE skill_scan_runs
             SET state = 'error', decision = 'block', progress = 100,
                 error_code = ?1, error_detail = ?2, completed_at = CURRENT_TIMESTAMP
             WHERE scan_id = ?3 AND state IN ('queued', 'scanning')",
            params![error_code, truncate(error_detail, 2048), scan_id],
        )?;
        Ok(())
    }

    pub fn cancel_scan(conn: &Connection, scan_id: &str) -> Result<bool> {
        Ok(conn.execute(
            "UPDATE skill_scan_runs
             SET state = 'cancelled', decision = 'block', progress = 100,
                 error_code = 'SCAN_CANCELLED', completed_at = CURRENT_TIMESTAMP
             WHERE scan_id = ?1 AND state IN ('queued', 'scanning')",
            [scan_id],
        )? > 0)
    }

    pub fn recover_interrupted(conn: &Connection) -> Result<usize> {
        conn.execute(
            "UPDATE skill_scan_runs
             SET state = 'error', decision = 'block', progress = 100,
                 error_code = 'SCAN_INTERRUPTED',
                 error_detail = 'Application stopped before the scan completed',
                 completed_at = CURRENT_TIMESTAMP
             WHERE state IN ('queued', 'scanning')",
            [],
        )
        .map_err(Into::into)
    }

    /// Retain current scans and recent audit history, then remove orphaned
    /// artifact rows. Finding and approval rows follow their scan by cascade.
    pub fn cleanup_history(conn: &Connection, retention_days: u32) -> Result<(usize, usize)> {
        let retention_days = retention_days.clamp(30, 3650);
        let transaction = conn.unchecked_transaction()?;
        let scans = transaction.execute(
            "DELETE FROM skill_scan_runs
             WHERE completed_at IS NOT NULL
               AND datetime(completed_at) < datetime('now', '-' || ?1 || ' days')
               AND scan_id NOT IN (
                   SELECT current_scan_id FROM skill_sources WHERE current_scan_id IS NOT NULL
               )",
            [retention_days],
        )?;
        let artifacts = transaction.execute(
            "DELETE FROM skill_artifacts
             WHERE artifact_hash NOT IN (SELECT artifact_hash FROM skill_scan_runs)
               AND artifact_hash NOT IN (SELECT artifact_hash FROM skill_sources)",
            [],
        )?;
        transaction.commit()?;
        Ok((scans, artifacts))
    }

    pub fn attach_scan_to_source(
        conn: &Connection,
        skill_id: &str,
        scan_id: &str,
        security_state: &str,
    ) -> Result<u64> {
        let deactivate = !matches!(security_state, "passed" | "warnings" | "approved");
        let changed = conn.execute(
            "UPDATE skill_sources
             SET current_scan_id = ?1, security_state = ?2,
                 user_enabled = CASE WHEN ?3 THEN 0 ELSE user_enabled END,
                 disabled_reason = CASE
                    WHEN ?3 THEN ?2
                    WHEN disabled_reason IN ('unscanned', 'stale', 'review_required', 'blocked', 'scan_error')
                      THEN NULL
                    ELSE disabled_reason
                 END,
                 updated_at = CURRENT_TIMESTAMP
             WHERE skill_id = ?4",
            params![scan_id, security_state, deactivate, skill_id],
        )?;
        if changed != 1 {
            bail!("Skill source is not registered")
        }
        SkillSourceRepo::bump_generation(conn)
    }

    pub fn decision_for_source(
        conn: &Connection,
        skill_id: &str,
    ) -> Result<Option<ScanDecisionRecord>> {
        conn.query_row(
            "SELECT scan.scan_id, scan.artifact_hash, scan.state, scan.decision,
                    scan.engine_versions_json, scan.policy_version
             FROM skill_sources source
             JOIN skill_scan_runs scan ON scan.scan_id = source.current_scan_id
             WHERE source.skill_id = ?1 AND scan.artifact_hash = source.artifact_hash",
            [skill_id],
            |row| {
                Ok(ScanDecisionRecord {
                    scan_id: row.get(0)?,
                    artifact_hash: row.get(1)?,
                    state: row.get(2)?,
                    decision: row.get(3)?,
                    engine_versions_json: row.get(4)?,
                    policy_version: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn scan_decision(conn: &Connection, scan_id: &str) -> Result<ScanDecisionRecord> {
        conn.query_row(
            "SELECT scan_id, artifact_hash, state, decision, engine_versions_json, policy_version
             FROM skill_scan_runs WHERE scan_id = ?1",
            [scan_id],
            |row| {
                Ok(ScanDecisionRecord {
                    scan_id: row.get(0)?,
                    artifact_hash: row.get(1)?,
                    state: row.get(2)?,
                    decision: row.get(3)?,
                    engine_versions_json: row.get(4)?,
                    policy_version: row.get(5)?,
                })
            },
        )
        .optional()?
        .context("Skill scan does not exist")
    }

    pub fn source_id_for_scan(conn: &Connection, scan_id: &str) -> Result<Option<String>> {
        conn.query_row(
            "SELECT skill_id FROM skill_sources WHERE current_scan_id = ?1 LIMIT 1",
            [scan_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn has_active_approval(
        conn: &Connection,
        artifact_hash: &str,
        scan_id: &str,
    ) -> Result<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM skill_approvals
             WHERE artifact_hash = ?1 AND scan_id = ?2 AND decision = 'approve'
               AND revoked_at IS NULL
               AND (expires_at IS NULL OR datetime(expires_at) > CURRENT_TIMESTAMP)",
            params![artifact_hash, scan_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn override_scan_decision(
        conn: &Connection,
        scan_id: &str,
        state: &str,
        decision: &str,
    ) -> Result<()> {
        let changed = conn.execute(
            "UPDATE skill_scan_runs SET state = ?1, decision = ?2
             WHERE scan_id = ?3 AND state = 'review_required'",
            params![state, decision, scan_id],
        )?;
        if changed != 1 {
            bail!("Only a review-required scan can be overridden")
        }
        Ok(())
    }

    pub fn mark_source_stale(conn: &Connection, skill_id: &str, reason: &str) -> Result<u64> {
        let transaction = conn.unchecked_transaction()?;
        transaction.execute(
            "UPDATE skill_scan_runs SET state = 'stale', decision = 'block'
             WHERE scan_id = (
                SELECT current_scan_id FROM skill_sources WHERE skill_id = ?1
             ) AND state NOT IN ('stale', 'error', 'cancelled')",
            [skill_id],
        )?;
        let changed = transaction.execute(
            "UPDATE skill_sources
             SET security_state = 'stale', user_enabled = 0,
                 disabled_reason = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE skill_id = ?2",
            params![reason, skill_id],
        )?;
        if changed != 1 {
            bail!("Skill source is not registered")
        }
        transaction.execute(
            "UPDATE skill_activation_state
             SET generation = generation + 1, updated_at = CURRENT_TIMESTAMP
             WHERE singleton = 1",
            [],
        )?;
        transaction.commit()?;
        SkillSourceRepo::generation(conn)
    }

    pub fn expire_approvals(conn: &Connection) -> Result<bool> {
        let changed = conn.execute(
            "UPDATE skill_sources
             SET security_state = 'stale', user_enabled = 0,
                 disabled_reason = 'approval_expired', updated_at = CURRENT_TIMESTAMP
             WHERE security_state = 'approved'
               AND current_scan_id IS NOT NULL
               AND NOT EXISTS (
                   SELECT 1 FROM skill_approvals approval
                   JOIN skill_scan_runs scan ON scan.scan_id = approval.scan_id
                   WHERE scan.scan_id = skill_sources.current_scan_id
                     AND approval.artifact_hash = skill_sources.artifact_hash
                     AND approval.decision = 'approve'
                     AND approval.revoked_at IS NULL
                     AND (approval.expires_at IS NULL OR datetime(approval.expires_at) > CURRENT_TIMESTAMP)
               )",
            [],
        )?;
        if changed > 0 {
            SkillSourceRepo::bump_generation(conn)?;
        }
        Ok(changed > 0)
    }

    pub fn scan_summary(
        conn: &Connection,
        skill_id: &str,
        generation: u64,
    ) -> Result<SkillScanSummary> {
        let row = conn
            .query_row(
                "SELECT scan.scan_id, scan.state, scan.decision, scan.max_severity,
                        scan.engine_versions_json, scan.policy_version, scan.completed_at
                 FROM skill_sources source
                 JOIN skill_scan_runs scan ON scan.scan_id = source.current_scan_id
                 WHERE source.skill_id = ?1",
                [skill_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                    ))
                },
            )
            .optional()?;
        let Some((scan_id, state, decision, max_severity, engines, policy, completed)) = row else {
            return Ok(SkillScanSummary {
                skill_id: skill_id.to_string(),
                generation,
                scan_id: None,
                state: "unscanned".to_string(),
                decision: None,
                max_severity: None,
                finding_counts: BTreeMap::new(),
                engine_version: None,
                policy_version: None,
                last_scanned_at: None,
                placeholder: false,
            });
        };
        let mut counts = BTreeMap::new();
        let mut statement = conn.prepare(
            "SELECT severity, COUNT(*) FROM skill_findings
             WHERE scan_id = ?1 GROUP BY severity",
        )?;
        for row in statement.query_map([&scan_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
        })? {
            let (severity, count) = row?;
            counts.insert(severity, count);
        }
        Ok(SkillScanSummary {
            skill_id: skill_id.to_string(),
            generation,
            scan_id: Some(scan_id),
            state,
            decision,
            max_severity,
            finding_counts: counts,
            engine_version: Some(engines),
            policy_version: Some(policy),
            last_scanned_at: completed,
            placeholder: false,
        })
    }

    pub fn list_findings(
        conn: &Connection,
        scan_id: &str,
        severity: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<SkillFindingPage> {
        let limit = limit.clamp(1, 100);
        let cursor = cursor.unwrap_or("");
        let severity = severity.unwrap_or("");
        let mut statement = conn.prepare(
            "SELECT finding_id, scan_id, engine, rule_id, severity, category,
                    file_path, line_start, line_end, title, detail, remediation,
                    fingerprint, evidence_redacted
             FROM skill_findings
             WHERE scan_id = ?1 AND finding_id > ?2
               AND (?3 = '' OR severity = ?3)
             ORDER BY finding_id LIMIT ?4",
        )?;
        let rows = statement
            .query_map(params![scan_id, cursor, severity, limit + 1], map_finding)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let has_more = rows.len() > limit as usize;
        let mut items = rows.into_iter().take(limit as usize).collect::<Vec<_>>();
        let next_cursor = has_more
            .then(|| items.last().map(|item| item.finding_id.clone()))
            .flatten();
        Ok(SkillFindingPage {
            scan_id: scan_id.to_string(),
            items: std::mem::take(&mut items),
            next_cursor,
        })
    }

    pub fn get_finding(conn: &Connection, scan_id: &str, finding_id: &str) -> Result<SkillFinding> {
        conn.query_row(
            "SELECT finding_id, scan_id, engine, rule_id, severity, category,
                    file_path, line_start, line_end, title, detail, remediation,
                    fingerprint, evidence_redacted
             FROM skill_findings WHERE scan_id = ?1 AND finding_id = ?2",
            params![scan_id, finding_id],
            map_finding,
        )
        .optional()?
        .context("Finding does not exist")
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_approval(
        conn: &Connection,
        artifact_hash: &str,
        scan_id: &str,
        subject: &str,
        decision: &str,
        actor: &str,
        reason: &str,
        scope: &str,
        expires_at: Option<&str>,
        correlation_id: &str,
    ) -> Result<SkillApprovalRecord> {
        if actor.trim().is_empty() || reason.trim().len() < 3 {
            bail!("Approval actor and a meaningful reason are required")
        }
        if let Some(expires_at) = expires_at {
            let parsed = chrono::DateTime::parse_from_rfc3339(expires_at)
                .context("Approval expiration must be RFC 3339")?;
            if parsed <= chrono::Utc::now() {
                bail!("Approval expiration must be in the future")
            }
        }
        let approval_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO skill_approvals (
                approval_id, artifact_hash, scan_id, subject, decision, actor,
                reason, scope, expires_at, correlation_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                approval_id,
                artifact_hash,
                scan_id,
                subject,
                decision,
                actor.trim(),
                reason.trim(),
                scope,
                expires_at,
                correlation_id,
            ],
        )?;
        Self::approval(conn, &approval_id)
    }

    pub fn revoke_approval(conn: &Connection, approval_id: &str) -> Result<()> {
        let changed = conn.execute(
            "UPDATE skill_approvals SET revoked_at = CURRENT_TIMESTAMP
             WHERE approval_id = ?1 AND revoked_at IS NULL",
            [approval_id],
        )?;
        if changed != 1 {
            bail!("Approval is missing or already revoked")
        }
        Ok(())
    }

    pub fn list_approvals(conn: &Connection, scan_id: &str) -> Result<Vec<SkillApprovalRecord>> {
        let mut statement = conn.prepare(
            "SELECT approval_id, artifact_hash, scan_id, subject, decision, actor,
                    reason, scope, expires_at, revoked_at, correlation_id, created_at
             FROM skill_approvals WHERE scan_id = ?1
             ORDER BY created_at DESC, approval_id DESC",
        )?;
        let approvals = statement
            .query_map([scan_id], map_approval)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(anyhow::Error::from)?;
        Ok(approvals)
    }

    fn approval(conn: &Connection, approval_id: &str) -> Result<SkillApprovalRecord> {
        conn.query_row(
            "SELECT approval_id, artifact_hash, scan_id, subject, decision, actor,
                    reason, scope, expires_at, revoked_at, correlation_id, created_at
             FROM skill_approvals WHERE approval_id = ?1",
            [approval_id],
            map_approval,
        )
        .map_err(Into::into)
    }
}

fn map_approval(row: &Row<'_>) -> rusqlite::Result<SkillApprovalRecord> {
    Ok(SkillApprovalRecord {
        approval_id: row.get(0)?,
        artifact_hash: row.get(1)?,
        scan_id: row.get(2)?,
        subject: row.get(3)?,
        decision: row.get(4)?,
        actor: row.get(5)?,
        reason: row.get(6)?,
        scope: row.get(7)?,
        expires_at: row.get(8)?,
        revoked_at: row.get(9)?,
        correlation_id: row.get(10)?,
        created_at: row.get(11)?,
    })
}

fn map_finding(row: &Row<'_>) -> rusqlite::Result<SkillFinding> {
    Ok(SkillFinding {
        finding_id: row.get(0)?,
        scan_id: row.get(1)?,
        engine: row.get(2)?,
        rule_id: row.get(3)?,
        severity: row.get(4)?,
        category: row.get(5)?,
        file_path: row.get(6)?,
        line_start: row.get(7)?,
        line_end: row.get(8)?,
        title: row.get(9)?,
        detail: row.get(10)?,
        remediation: row.get(11)?,
        fingerprint: row.get(12)?,
        evidence_redacted: row.get(13)?,
    })
}

fn truncate(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::db::migrations::run_migrations;

    use super::SkillSecurityRepo;

    #[test]
    fn interrupted_scans_fail_closed_on_restart() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        SkillSecurityRepo::upsert_artifact(&conn, "hash", 1, "{}", None, "quarantined").unwrap();
        SkillSecurityRepo::create_scan(&conn, "scan", "hash", "{}", "policy", "corr").unwrap();
        SkillSecurityRepo::begin_scan(&conn, "scan").unwrap();

        assert_eq!(SkillSecurityRepo::recover_interrupted(&conn).unwrap(), 1);
        let state: String = conn
            .query_row(
                "SELECT state FROM skill_scan_runs WHERE scan_id = 'scan'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "error");
    }

    #[test]
    fn cleanup_keeps_current_scan_and_removes_only_expired_orphans() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        for (artifact, scan) in [("old-hash", "old-scan"), ("recent-hash", "recent-scan")] {
            SkillSecurityRepo::upsert_artifact(&conn, artifact, 1, "{}", None, "blocked").unwrap();
            SkillSecurityRepo::create_scan(&conn, scan, artifact, "{}", "policy", "corr").unwrap();
            SkillSecurityRepo::begin_scan(&conn, scan).unwrap();
            SkillSecurityRepo::complete_scan(&conn, scan, "blocked", "block", None, &[]).unwrap();
        }
        conn.execute(
            "UPDATE skill_scan_runs SET completed_at = '2000-01-01 00:00:00'
             WHERE scan_id = 'old-scan'",
            [],
        )
        .unwrap();

        assert_eq!(
            SkillSecurityRepo::cleanup_history(&conn, 90).unwrap(),
            (1, 1)
        );
        assert!(SkillSecurityRepo::scan_decision(&conn, "old-scan").is_err());
        assert!(SkillSecurityRepo::scan_decision(&conn, "recent-scan").is_ok());
    }
}
