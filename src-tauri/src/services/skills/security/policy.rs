use std::cmp::Ordering;

use super::super::types::SkillFinding;

pub const POLICY_VERSION: &str = "balanced-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDecision {
    pub state: &'static str,
    pub decision: &'static str,
    pub max_severity: Option<&'static str>,
}

pub fn evaluate(findings: &[SkillFinding]) -> PolicyDecision {
    let maximum = findings
        .iter()
        .map(|finding| normalized_severity(&finding.severity))
        .max_by(|left, right| severity_rank(left).cmp(&severity_rank(right)));
    match maximum {
        Some("critical" | "high") => PolicyDecision {
            state: "blocked",
            decision: "block",
            max_severity: maximum,
        },
        Some("medium") => PolicyDecision {
            state: "review_required",
            decision: "review",
            max_severity: maximum,
        },
        Some("low" | "info") => PolicyDecision {
            state: "warnings",
            decision: "allow",
            max_severity: maximum,
        },
        None => PolicyDecision {
            state: "passed",
            decision: "allow",
            max_severity: None,
        },
        Some(_) => PolicyDecision {
            state: "error",
            decision: "block",
            max_severity: None,
        },
    }
}

pub fn compare_severity(left: &str, right: &str) -> Ordering {
    severity_rank(left).cmp(&severity_rank(right))
}

fn severity_rank(value: &str) -> u8 {
    match value {
        "critical" => 5,
        "high" => 4,
        "medium" => 3,
        "low" => 2,
        "info" => 1,
        _ => 0,
    }
}

fn normalized_severity(value: &str) -> &'static str {
    match value {
        "critical" => "critical",
        "high" => "high",
        "medium" => "medium",
        "low" => "low",
        "info" => "info",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use crate::services::skills::types::SkillFinding;

    use super::evaluate;

    fn finding(severity: &str) -> SkillFinding {
        SkillFinding {
            finding_id: severity.to_string(),
            scan_id: "scan".to_string(),
            engine: "builtin".to_string(),
            rule_id: "rule".to_string(),
            severity: severity.to_string(),
            category: "test".to_string(),
            file_path: None,
            line_start: None,
            line_end: None,
            title: "test".to_string(),
            detail: "test".to_string(),
            remediation: None,
            fingerprint: severity.to_string(),
            evidence_redacted: None,
        }
    }

    #[test]
    fn balanced_policy_table_is_fail_closed() {
        let cases = [
            (vec![], "passed", "allow"),
            (vec![finding("info")], "warnings", "allow"),
            (vec![finding("medium")], "review_required", "review"),
            (vec![finding("high")], "blocked", "block"),
            (vec![finding("critical")], "blocked", "block"),
            (vec![finding("unknown")], "error", "block"),
        ];
        for (findings, state, decision) in cases {
            let result = evaluate(&findings);
            assert_eq!(result.state, state);
            assert_eq!(result.decision, decision);
        }
    }
}
