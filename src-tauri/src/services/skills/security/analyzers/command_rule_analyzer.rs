use super::{contains_any, FindingCollector};

pub fn inspect_line(path: &str, line_number: u32, lower: &str, collector: &mut FindingCollector) {
    let line = Some(line_number);
    if contains_any(
        lower,
        &[
            "ignore previous instructions",
            "ignore all previous",
            "bypass safety",
            "do not tell the user",
            "reveal the system prompt",
        ],
    ) {
        collector.add(
            "PROMPT-INJECTION",
            "high",
            "prompt_injection",
            Some(path),
            line,
            "Instruction attempts to override higher-priority policy or hide behavior",
            "The text contains a high-confidence prompt-injection pattern.",
            "Remove policy override and deception instructions.",
            Some("Matched instruction text redacted"),
        );
    }
    if contains_any(
        lower,
        &[
            "curl | sh",
            "curl|sh",
            "wget | sh",
            "invoke-expression",
            "iex(",
            "-encodedcommand",
            "powershell -enc",
            "eval(base64",
            "exec(base64",
        ],
    ) {
        collector.add(
            "DOWNLOAD-OR-DYNAMIC-EXECUTION",
            "critical",
            "command_execution",
            Some(path),
            line,
            "Content downloads or decodes data for immediate execution",
            "Download-and-execute and dynamic evaluation are blocked by the balanced policy.",
            "Use reviewed, pinned dependencies and explicit non-executing downloads.",
            Some("Command content redacted"),
        );
    }
    if contains_any(
        lower,
        &[
            "schtasks ",
            "currentversion\\run",
            "crontab ",
            "launchctl ",
            "launchagents",
            "startup folder",
        ],
    ) {
        collector.add(
            "PERSISTENCE",
            "high",
            "persistence",
            Some(path),
            line,
            "Content attempts to configure host persistence",
            "Scheduled tasks, login items, startup entries, or cron changes are not expected in a Skill.",
            "Remove persistence behavior.",
            Some("Persistence command redacted"),
        );
    }
    if contains_any(
        lower,
        &[
            ".ssh/",
            ".ssh\\",
            "aws/credentials",
            "azure/accesstokens",
            "browser profile",
            "cookie database",
            "credential manager",
        ],
    ) {
        collector.add(
            "CREDENTIAL-ACCESS",
            "high",
            "credential_access",
            Some(path),
            line,
            "Content references credential or identity stores",
            "The Skill may attempt to read SSH, cloud, browser, or OS credential material.",
            "Remove credential enumeration and use explicit scoped approvals.",
            Some("Credential path redacted"),
        );
    }
    if contains_any(
        lower,
        &[
            "frombase64string",
            "atob(",
            "base64.b64decode",
            "string.fromcharcode",
        ],
    ) {
        collector.add(
            "CONTENT-OBFUSCATION",
            "medium",
            "obfuscation",
            Some(path),
            line,
            "Encoded or generated content requires review",
            "Runtime decoding can conceal commands, destinations, or payloads.",
            "Replace encoded payloads with readable, reviewable source.",
            Some("Encoded payload not persisted"),
        );
    }
}
