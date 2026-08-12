//! Format detection from content or file extension.

use crate::ImportFormat;

/// Map a file extension (no leading dot) to an import format.
#[must_use]
pub fn format_from_extension(ext: &str) -> Option<ImportFormat> {
    match ext
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "cho" | "crd" | "chopro" | "chordpro" | "pro" => Some(ImportFormat::ChordPro),
        "txt" | "text" => Some(ImportFormat::PlainText),
        _ => None,
    }
}

/// Heuristic detection. Bracketed section headers like `[Chorus]` are not ChordPro.
#[must_use]
pub fn detect_format(input: &str) -> ImportFormat {
    if looks_like_chordpro(input) {
        ImportFormat::ChordPro
    } else {
        ImportFormat::PlainText
    }
}

fn looks_like_chordpro(input: &str) -> bool {
    input.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with('{') || has_inline_chordpro_chord(trimmed)
    })
}

fn has_inline_chordpro_chord(line: &str) -> bool {
    let mut search = line;
    while let Some(start) = search.find('[') {
        let after_open = &search[start + 1..];
        let Some(end) = after_open.find(']') else {
            break;
        };
        let inner = after_open[..end].trim();
        let after = after_open[end + 1..].chars().next();
        if !inner.is_empty()
            && !is_plain_section_name(inner)
            && after.is_some_and(|c| !c.is_whitespace())
        {
            return true;
        }
        search = &after_open[end + 1..];
    }
    false
}

fn is_plain_section_name(inner: &str) -> bool {
    let lower = inner.to_ascii_lowercase();
    let word = lower
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|c: char| !c.is_ascii_alphabetic());
    matches!(
        word,
        "verse"
            | "chorus"
            | "bridge"
            | "intro"
            | "outro"
            | "solo"
            | "instrumental"
            | "prechorus"
            | "pre"
            | "tag"
            | "interlude"
            | "breakdown"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_mapping() {
        assert_eq!(format_from_extension("cho"), Some(ImportFormat::ChordPro));
        assert_eq!(format_from_extension(".CRD"), Some(ImportFormat::ChordPro));
        assert_eq!(format_from_extension("txt"), Some(ImportFormat::PlainText));
        assert_eq!(format_from_extension("pdf"), None);
    }

    #[test]
    fn detects_chordpro_vs_plain() {
        assert_eq!(detect_format("{title: X}\n[C]Hi"), ImportFormat::ChordPro);
        assert_eq!(detect_format("[C]Hello [G]world"), ImportFormat::ChordPro);
        assert_eq!(
            detect_format("[Chorus]\nC     G\nHello there"),
            ImportFormat::PlainText
        );
        assert_eq!(
            detect_format("C          G\nAmazing grace"),
            ImportFormat::PlainText
        );
    }
}
