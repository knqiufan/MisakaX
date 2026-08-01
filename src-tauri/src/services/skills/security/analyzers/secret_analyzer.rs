use super::{contains_any, FindingCollector};

pub fn inspect_line(
    path: &str,
    line_number: u32,
    original: &str,
    lower: &str,
    collector: &mut FindingCollector,
) {
    if looks_like_secret(original, lower) {
        collector.add(
            "EMBEDDED-SECRET",
            "high",
            "secret",
            Some(path),
            Some(line_number),
            "Potential embedded credential or secret",
            "A high-confidence credential-shaped value was detected. The value was not stored.",
            "Remove the value, rotate it, and use a scoped secret provider.",
            Some("[REDACTED]"),
        );
    }
}

fn looks_like_secret(original: &str, lower: &str) -> bool {
    if original.split_whitespace().any(|word| {
        word.starts_with("AKIA")
            && word
                .chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .count()
                >= 20
    }) {
        return true;
    }
    let secret_name = contains_any(
        lower,
        &[
            "api_key",
            "apikey",
            "client_secret",
            "private_key",
            "password",
        ],
    );
    let assignment = original.contains('=') || original.contains(':');
    let quoted_payload = original
        .split(['=', ':'])
        .nth(1)
        .map(str::trim)
        .map(|value| value.trim_matches(['\'', '"']).len() >= 16)
        .unwrap_or(false);
    secret_name && assignment && quoted_payload
}
