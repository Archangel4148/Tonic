//! Shared section-header and layout-marker helpers.

use tonic_domain::{parse_chord, Capo, ParseStatus, SectionLabel};

/// First capo fret (`1..=12`) mentioned in `value` (`2`, `2nd fret`, …).
#[must_use]
pub fn parse_capo_fret(value: &str) -> Option<u8> {
    let digits: String = value
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let fret = digits.parse().ok()?;
    let capo = Capo::new(fret).ok()?;
    (capo.fret() > 0).then_some(capo.fret())
}

/// If `line` starts with `[Intro]` / `[Chorus]` / …, return that label and the rest.
///
/// Unknown `[Name]` headings become a custom section so the editor can round-trip
/// labels the built-in list does not know.
#[must_use]
pub fn split_leading_section_header(line: &str) -> Option<(SectionLabel, &str)> {
    let trimmed = line.trim();
    if let Some(stripped) = trimmed.strip_prefix('[') {
        let end = stripped.find(']')?;
        let inner = stripped[..end].trim();
        if inner.is_empty() {
            return None;
        }
        let rest = stripped[end + 1..].trim();
        if let Some(label) = parse_section_header(inner) {
            return Some((label, rest));
        }
        // ChordPro `[C]lyric` must not become a section named "C".
        if is_chord_like_bracket(inner) {
            return None;
        }
        return Some((
            SectionLabel::Custom {
                name: inner.to_string(),
            },
            rest,
        ));
    }
    parse_section_header(trimmed).map(|label| (label, ""))
}

fn is_chord_like_bracket(inner: &str) -> bool {
    if is_layout_marker(inner) || is_no_chord_mark(inner) || is_repeat_marker(inner) {
        return true;
    }
    !matches!(parse_chord(inner).status(), ParseStatus::Unrecognized)
}

/// `Verse 1`, `[Chorus]`, `[INTRO][:]`, `Pre-Chorus`, …
#[must_use]
pub fn parse_section_header(line: &str) -> Option<SectionLabel> {
    let stripped = line
        .trim()
        .trim_matches(|c: char| matches!(c, '[' | ']' | '*' | '#' | ':'))
        .trim();
    if stripped.is_empty() {
        return None;
    }
    let lower = stripped.to_ascii_lowercase();
    let mut parts = lower.split_whitespace();
    let word = parts.next()?;
    let number = parts.find_map(|p| p.parse::<u16>().ok());
    match word {
        "verse" => Some(SectionLabel::Verse { number }),
        "chorus" => Some(SectionLabel::Chorus { number }),
        "bridge" => Some(SectionLabel::Bridge),
        "intro" => Some(SectionLabel::Intro),
        "outro" => Some(SectionLabel::Outro),
        "solo" => Some(SectionLabel::Solo),
        "instrumental" | "interlude" => Some(SectionLabel::Instrumental),
        "pre-chorus" | "prechorus" => Some(SectionLabel::PreChorus),
        "pre" if lower.contains("chorus") => Some(SectionLabel::PreChorus),
        "tag" | "breakdown" | "harmonies" => Some(SectionLabel::Custom {
            name: stripped.to_string(),
        }),
        _ if lower.ends_with(" solo") => Some(SectionLabel::Solo),
        _ => None,
    }
}

/// Ultimate Guitar repeat shorthand: `x2`, `3x`, `(x4)`, …
#[must_use]
pub fn is_repeat_marker(token: &str) -> bool {
    let trimmed = token
        .trim()
        .trim_matches(|c: char| matches!(c, '(' | ')' | '[' | ']'));
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower
        .strip_prefix('x')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
        || lower
            .strip_suffix('x')
            .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

#[must_use]
pub fn is_no_chord_mark(token: &str) -> bool {
    matches!(
        token.trim().to_ascii_lowercase().as_str(),
        "n.c." | "nc" | "n/c" | "n.c" | "%"
    )
}

/// ChordPro bar / repeat / empty placeholders such as `[|]`, `[-]`, `[:]`.
#[must_use]
pub fn is_layout_marker(inner: &str) -> bool {
    matches!(
        inner.trim(),
        "" | "|" | "-" | "/" | ":" | "." | "||" | "||:" | ":||" | "(" | ")" | "*" | "**" | "***"
    ) || is_no_chord_mark(inner)
        || is_capo_fragment(inner)
        || is_repeat_marker(inner)
}

fn is_capo_fragment(inner: &str) -> bool {
    let token = inner
        .trim()
        .trim_matches(|c: char| matches!(c, '(' | ')'))
        .trim()
        .to_ascii_lowercase();
    token.starts_with("capo")
        || (token.starts_with('+') && token[1..].chars().all(|c| c.is_ascii_digit()))
}

/// `[(capo][+1)]` or `[capo 1]` → inline label (`Capo +1` / `Capo 1`).
#[must_use]
pub fn extract_capo_directive(line: &str) -> Option<String> {
    let collapsed: String = line
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '+' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let lower = collapsed.to_ascii_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();
    if !parts.contains(&"capo") {
        return None;
    }
    let only_capo = parts.iter().all(|part| {
        *part == "capo"
            || part
                .trim_start_matches('+')
                .chars()
                .all(|c| c.is_ascii_digit())
    });
    if !only_capo {
        return None;
    }
    let relative = parts
        .iter()
        .any(|part| part.starts_with('+') && part[1..].chars().all(|c| c.is_ascii_digit()));
    let fret = parts
        .iter()
        .find_map(|part| part.trim_start_matches('+').parse::<u8>().ok())?;
    Some(if relative {
        format!("Capo +{fret}")
    } else {
        format!("Capo {fret}")
    })
}

#[must_use]
pub fn is_prose_annotation(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    lower.starts_with("tip:")
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("www.")
}

/// Non-lyric chart note: capo hints, repeat counts, transcriber credit, …
#[must_use]
pub fn is_chart_annotation(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    is_prose_annotation(trimmed)
        || lower.starts_with("tabbed by")
        || is_inline_capo_line(trimmed)
        || lower.starts_with("original key")
        || lower.starts_with("repeat ")
        || is_repeat_marker(trimmed)
        || lower.starts_with("(transpose")
        || lower.starts_with("the first few lines")
        || lower.starts_with("transpose ")
}

#[must_use]
pub fn is_inline_capo_line(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    if !lower.starts_with("capo") {
        return false;
    }
    let rest = lower[4..].trim();
    rest.is_empty()
        || rest
            .chars()
            .all(|c| c.is_whitespace() || c.is_ascii_digit())
        || matches!(
            rest,
            "i" | "ii" | "iii" | "iv" | "v" | "vi" | "vii" | "viii" | "ix" | "x" | "xi" | "xii"
        )
}
