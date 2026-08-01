use super::FindingCollector;

pub fn inspect_path(path: &str, collector: &mut FindingCollector) {
    if path.chars().any(|character| {
        matches!(
            character,
            '\u{061c}'
                | '\u{200b}'
                | '\u{200c}'
                | '\u{200d}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{2069}'
                | '\u{feff}'
        )
    }) {
        collector.add(
            "UNICODE-CONTROL",
            "high",
            "obfuscation",
            Some(path),
            None,
            "Path contains bidirectional or invisible control characters",
            "A file path can render differently from its actual value.",
            "Rename the file using visible Unicode characters only.",
            Some("Control characters redacted"),
        );
    }
}

pub fn inspect_file_signature(
    path: &str,
    extension: &str,
    bytes: &[u8],
    collector: &mut FindingCollector,
) {
    let zip_magic = bytes.starts_with(b"PK\x03\x04");
    if zip_magic {
        collector.add(
            "NESTED-ARCHIVE",
            "medium",
            "nested_archive",
            Some(path),
            None,
            "Nested archive requires explicit review",
            "Nested archives are inventoried but not recursively extracted by the baseline scanner.",
            "Unpack required source files into the top-level Skill artifact.",
            None,
        );
    }
    let expected_zip = matches!(extension, "zip" | "jar" | "docx" | "xlsx" | "pptx");
    if zip_magic && !expected_zip {
        collector.add(
            "MIME-EXTENSION-MISMATCH",
            "low",
            "content_inventory",
            Some(path),
            None,
            "File signature does not match its extension",
            "The file has a ZIP signature but its extension does not identify an archive container.",
            "Use a truthful file extension and document why the container is needed.",
            None,
        );
    }
}

pub fn is_executable(bytes: &[u8]) -> bool {
    bytes.starts_with(b"MZ")
        || bytes.starts_with(b"\x7fELF")
        || bytes.starts_with(&[0xfe, 0xed, 0xfa, 0xce])
        || bytes.starts_with(&[0xcf, 0xfa, 0xed, 0xfe])
}
