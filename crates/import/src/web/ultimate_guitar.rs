//! Ultimate Guitar chord-tab adapter.
//!
//! UG embeds page data in `<div class="js-store" data-content="…JSON…">`.
//! Chord bodies use `[ch]Am[/ch]` markers. When wrapped in `[tab]` blocks, chords
//! sit on a spaced line above lyrics (plain-text layout). Otherwise chords are
//! inline on the lyric line (ChordPro layout).

use serde_json::Value;
use tonic_domain::{Key, SectionLabel, SongId, SongSource};

use crate::chordpro::import_chordpro;
use crate::plain::import_plain_text;
use crate::section::{is_chart_annotation, is_repeat_marker, split_leading_section_header};
use crate::warning::{ImportWarning, WarningKind};
use crate::{ImportResult, WebImportError};

pub const ULTIMATE_GUITAR_SITE: &str = "ultimate-guitar";

#[must_use]
pub fn matches_ultimate_guitar_url(url: &str) -> bool {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    let host_ok = lower.contains("tabs.ultimate-guitar.com/")
        || lower.contains("www.ultimate-guitar.com/")
        || lower.contains("//ultimate-guitar.com/");
    host_ok && lower.contains("/tab/") && lower.contains("-chords-")
}

/// Parse an Ultimate Guitar HTML document into a song.
pub fn parse_ultimate_guitar_html(
    url: &str,
    html: &str,
    id: impl Into<SongId>,
) -> Result<ImportResult, WebImportError> {
    if html_looks_blocked(html) {
        return Err(WebImportError::ParseFailed(
            "Ultimate Guitar blocked the page request (bot protection). Try again later, or paste the chart text instead."
                .to_string(),
        ));
    }

    let data_content = extract_js_store(html).ok_or_else(|| {
        WebImportError::ParseFailed(
            "Could not find song data on this Ultimate Guitar page. Open a chords tab URL, or paste the chart text."
                .to_string(),
        )
    })?;

    let json: Value = serde_json::from_str(&data_content).map_err(|error| {
        WebImportError::ParseFailed(format!(
            "Ultimate Guitar page data was corrupt or unexpected ({error})."
        ))
    })?;

    let tab = json
        .pointer("/store/page/data/tab")
        .or_else(|| find_object_with_keys(&json, &["song_name", "artist_name"]));
    let tab_view = json
        .pointer("/store/page/data/tab_view")
        .or_else(|| json.pointer("/store/page/data"));

    let song_name = tab
        .and_then(|value| value.get("song_name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let artist_name = tab
        .and_then(|value| value.get("artist_name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let content = tab_view
        .and_then(|view| view.pointer("/wiki_tab/content"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            WebImportError::ParseFailed(
                "This Ultimate Guitar page has no chord chart text (guitar pro / video tabs are not supported)."
                    .to_string(),
            )
        })?;

    let capo = tab_view
        .and_then(|view| view.pointer("/meta/capo"))
        .and_then(Value::as_u64)
        .or_else(|| {
            tab_view
                .and_then(|view| view.get("capo"))
                .and_then(Value::as_u64)
        });
    let tonality = tab_view
        .and_then(|view| view.pointer("/meta/tonality"))
        .and_then(Value::as_str)
        .or_else(|| {
            tab.and_then(|value| value.get("tonality_name"))
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let decoded = decode_html_entities(content);
    let body = normalize_line_breaks(&decoded);
    let uses_tab_blocks = body.to_ascii_lowercase().contains("[tab]");

    let mut result = if uses_tab_blocks {
        let plain = ug_tab_content_to_plain(
            &body,
            song_name,
            artist_name,
            tonality,
        );
        import_plain_text(&plain, id)
    } else {
        let chart = ug_inline_content_to_chordpro(
            &body,
            song_name,
            artist_name,
            tonality,
            capo,
        );
        import_chordpro(&chart, id)
    };

    result.song.set_source(SongSource::web(
        url.trim(),
        ULTIMATE_GUITAR_SITE,
        Some(content.to_string()),
    ));

    if result.song.title() == "Untitled" {
        if let Some(name) = song_name {
            result.song.set_title(name);
        }
    }
    if result.song.artist().is_none() {
        if let Some(artist) = artist_name {
            result.song.set_artist(Some(artist.to_string()));
        }
    }
    if result.song.original_key().is_none() {
        if let Some(symbol) = tonality {
            if let Some(key) = Key::parse(symbol) {
                result.song.set_original_key(Some(key));
                result.song.set_performance_key(Some(key));
            }
        }
    }

    if result.song.sections().is_empty() {
        result.warnings.push(ImportWarning::new(
            WarningKind::MalformedInput,
            "Ultimate Guitar chart had no lyric or chord lines after conversion.",
            None,
        ));
    }

    Ok(result)
}

fn html_looks_blocked(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    lower.contains("cf-challenge")
        || lower.contains("just a moment")
        || lower.contains("access denied")
        || (lower.contains("cloudflare") && !lower.contains("js-store"))
}

fn extract_js_store(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let marker = "js-store";
    let mut search_from = 0;
    while let Some(rel) = lower[search_from..].find(marker) {
        let abs = search_from + rel;
        let tag_start = html[..abs].rfind('<')?;
        let tag_end = html[abs..].find('>')? + abs;
        let tag = &html[tag_start..=tag_end];
        if let Some(raw) = attribute_value(tag, "data-content") {
            let decoded = decode_html_entities(raw);
            if decoded.contains("wiki_tab")
                || decoded.contains("song_name")
                || decoded.contains("\"store\"")
            {
                return Some(decoded);
            }
        }
        search_from = abs + marker.len();
    }
    None
}

fn attribute_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let lower = tag.to_ascii_lowercase();
    let key = format!("{name}=");
    let idx = lower.find(&key)?;
    let after = &tag[idx + key.len()..];
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn decode_html_entities(input: &str) -> String {
    let mut out = input.to_string();
    for _ in 0..4 {
        let next = decode_html_entities_once(&out);
        if next == out {
            break;
        }
        out = next;
    }
    out
}

fn decode_html_entities_once(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != ';' && i - start < 12 {
                i += 1;
            }
            if i < chars.len() && chars[i] == ';' {
                let entity: String = chars[start..=i].iter().collect();
                if let Some(decoded) = decode_entity(&entity) {
                    out.push(decoded);
                    i += 1;
                    continue;
                }
            }
            out.push(chars[start]);
            i = start + 1;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn decode_entity(entity: &str) -> Option<char> {
    if !entity.starts_with('&') || !entity.ends_with(';') {
        return None;
    }
    let inner = &entity[1..entity.len() - 1];
    match inner {
        "quot" => Some('"'),
        "apos" => Some('\''),
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "nbsp" => Some('\u{00a0}'),
        _ if inner.starts_with('#') => {
            if inner.len() > 2 && (inner.as_bytes()[1] == b'x' || inner.as_bytes()[1] == b'X') {
                u32::from_str_radix(&inner[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
            } else {
                inner[1..].parse::<u32>().ok().and_then(char::from_u32)
            }
        }
        _ => None,
    }
}

fn normalize_line_breaks(content: &str) -> String {
    content
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<br>", "\n")
}

fn ug_tab_content_to_plain(
    content: &str,
    title: Option<&str>,
    artist: Option<&str>,
    key: Option<&str>,
) -> String {
    let mut out = String::new();
    if let Some(title) = title {
        out.push_str(&format!("title: {title}\n"));
    }
    if let Some(artist) = artist {
        out.push_str(&format!("artist: {artist}\n"));
    }
    if let Some(key) = key {
        out.push_str(&format!("key: {key}\n"));
    }
    out.push('\n');

    let expanded = expand_ug_tab_blocks(content);
    let mut pending_annotations: Vec<String> = Vec::new();

    for line in expanded.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if is_chart_annotation(trimmed) {
            pending_annotations.push(trimmed.to_string());
            continue;
        }
        if let Some((label, rest)) = split_leading_section_header(trimmed) {
            out.push_str(&section_label_line(&label));
            out.push('\n');
            flush_pending_annotations(&mut out, &mut pending_annotations);
            if !rest.is_empty() {
                if is_chart_annotation(rest) {
                    out.push_str(rest);
                    out.push('\n');
                } else {
                    for part in normalize_ug_plain_line(rest) {
                        out.push_str(&part);
                        out.push('\n');
                    }
                }
            }
            continue;
        }
        flush_pending_annotations(&mut out, &mut pending_annotations);
        for part in normalize_ug_plain_line(line.trim_end()) {
            out.push_str(&part);
            out.push('\n');
        }
    }
    flush_pending_annotations(&mut out, &mut pending_annotations);
    out
}

fn flush_pending_annotations(out: &mut String, pending: &mut Vec<String>) {
    for note in pending.drain(..) {
        out.push_str(&note);
        out.push('\n');
    }
}

fn normalize_ug_plain_line(line: &str) -> Vec<String> {
    let stripped = strip_ug_chord_markers(line);
    let (body, repeat) = split_trailing_repeat(&stripped);
    let mut out = Vec::new();
    if !body.trim().is_empty() {
        out.push(body);
    }
    if let Some(repeat) = repeat {
        out.push(repeat);
    }
    out
}

fn split_trailing_repeat(line: &str) -> (String, Option<String>) {
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        return (String::new(), None);
    }
    let Some((body, last)) = split_last_token(trimmed) else {
        return (trimmed.to_string(), None);
    };
    if is_repeat_marker(last) {
        let repeat = repeat_marker_count(last).map(|count| format!("Repeat {count}×"));
        return (body.trim_end().to_string(), repeat);
    }
    (trimmed.to_string(), None)
}

fn split_last_token(line: &str) -> Option<(&str, &str)> {
    let end = line.trim_end();
    let start = end
        .char_indices()
        .rfind(|(_, c)| c.is_whitespace())
        .map(|(idx, _)| idx)?;
    let body = &end[..start];
    let last = end[start..].trim_start();
    if last.is_empty() {
        return None;
    }
    Some((body, last))
}

fn repeat_marker_count(token: &str) -> Option<u32> {
    let trimmed = token
        .trim()
        .trim_matches(|c: char| matches!(c, '(' | ')' | '[' | ']'));
    let lower = trimmed.to_ascii_lowercase();
    if let Some(digits) = lower.strip_prefix('x') {
        return digits.parse().ok();
    }
    if let Some(digits) = lower.strip_suffix('x') {
        return digits.parse().ok();
    }
    None
}

fn section_label_line(label: &SectionLabel) -> String {
    match label {
        SectionLabel::Verse { number: Some(n) } => format!("[Verse {n}]"),
        SectionLabel::Verse { number: None } => "[Verse]".to_string(),
        SectionLabel::Chorus { number: Some(n) } => format!("[Chorus {n}]"),
        SectionLabel::Chorus { number: None } => "[Chorus]".to_string(),
        SectionLabel::Bridge => "[Bridge]".to_string(),
        SectionLabel::Intro => "[Intro]".to_string(),
        SectionLabel::Outro => "[Outro]".to_string(),
        SectionLabel::Solo => "[Solo]".to_string(),
        SectionLabel::PreChorus => "[Pre-Chorus]".to_string(),
        SectionLabel::Instrumental => "[Instrumental]".to_string(),
        SectionLabel::Custom { name } => format!("[{name}]"),
    }
}

fn expand_ug_tab_blocks(input: &str) -> String {
    let mut out = String::new();
    let lower = input.to_ascii_lowercase();
    let open = "[tab]";
    let close = "[/tab]";
    let mut i = 0;
    while i < input.len() {
        if lower[i..].starts_with(open) {
            let start = i + open.len();
            if let Some(rel) = lower[start..].find(close) {
                let body = &input[start..start + rel];
                if let Some(expanded) = expand_single_tab_block(body) {
                    out.push_str(&expanded);
                    out.push('\n');
                }
                i = start + rel + close.len();
                continue;
            }
        }
        let ch = input[i..].chars().next().unwrap_or('\0');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn expand_single_tab_block(body: &str) -> Option<String> {
    let mut lines: Vec<&str> = body.lines().collect();
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    if lines.is_empty() {
        return None;
    }
    if lines.len() == 1 {
        let chord_line = strip_ug_chord_markers(lines[0]);
        if chord_line.trim().is_empty() {
            return None;
        }
        return Some(chord_line);
    }
    let chord_line = strip_ug_chord_markers(lines[0]);
    let lyric_line = lines[1..].join("\n").trim_end().to_string();
    if chord_line.trim().is_empty() || lyric_line.is_empty() {
        return None;
    }
    Some(format!("{chord_line}\n{lyric_line}"))
}

fn strip_ug_chord_markers(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let lower = input.to_ascii_lowercase();
    let open = "[ch]";
    let close = "[/ch]";
    let mut i = 0;
    while i < input.len() {
        if lower[i..].starts_with(open) {
            let start = i + open.len();
            if let Some(rel) = lower[start..].find(close) {
                out.push_str(input[start..start + rel].trim());
                i = start + rel + close.len();
                continue;
            }
        }
        let ch = input[i..].chars().next().unwrap_or('\0');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn ug_inline_content_to_chordpro(
    content: &str,
    title: Option<&str>,
    artist: Option<&str>,
    key: Option<&str>,
    capo: Option<u64>,
) -> String {
    let mut out = String::new();
    if let Some(title) = title {
        out.push_str(&format!("{{title: {title}}}\n"));
    }
    if let Some(artist) = artist {
        out.push_str(&format!("{{artist: {artist}}}\n"));
    }
    if let Some(key) = key {
        out.push_str(&format!("{{key: {key}}}\n"));
    }
    if let Some(capo) = capo.filter(|value| *value > 0) {
        out.push_str(&format!("{{capo: {capo}}}\n"));
    }
    out.push('\n');

    let with_chords = replace_ug_chords(content);
    let without_tab = strip_tag_pairs(&with_chords, "tab");
    let normalized = without_tab
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<br>", "\n");
    out.push_str(&normalized);
    out
}

fn replace_ug_chords(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let lower = input.to_ascii_lowercase();
    let open = "[ch]";
    let close = "[/ch]";
    let mut i = 0;
    while i < input.len() {
        if lower[i..].starts_with(open) {
            let start = i + open.len();
            if let Some(rel) = lower[start..].find(close) {
                let chord = input[start..start + rel].trim();
                out.push('[');
                out.push_str(chord);
                out.push(']');
                i = start + rel + close.len();
                continue;
            }
        }
        let ch = input[i..].chars().next().unwrap_or('\0');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn strip_tag_pairs(input: &str, name: &str) -> String {
    let open = format!("[{name}]");
    let close = format!("[/{name}]");
    input
        .replace(&open, "")
        .replace(&close, "")
        .replace(&open.to_ascii_uppercase(), "")
        .replace(&close.to_ascii_uppercase(), "")
}

fn find_object_with_keys<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if keys.iter().all(|key| map.contains_key(*key)) {
                return Some(value);
            }
            for child in map.values() {
                if let Some(found) = find_object_with_keys(child, keys) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => {
            for item in items {
                if let Some(found) = find_object_with_keys(item, keys) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_chord_tab_urls() {
        assert!(matches_ultimate_guitar_url(
            "https://tabs.ultimate-guitar.com/tab/traditional/amazing-grace-chords-1080922"
        ));
        assert!(!matches_ultimate_guitar_url(
            "https://tabs.ultimate-guitar.com/tab/foo/bar-guitar-pro-123"
        ));
        assert!(!matches_ultimate_guitar_url("https://example.com/tab/x"));
    }

    #[test]
    fn decodes_numeric_html_entities() {
        assert_eq!(decode_html_entities("summer&#039;s evenin&#39;"), "summer's evenin'");
        assert_eq!(decode_html_entities("&amp;amp;"), "&");
    }

    #[test]
    fn expands_tab_blocks_to_chord_over_lyrics() {
        let input = "[tab]     [ch]G[/ch]                          [ch]C[/ch]               [ch]G[/ch]\nOn a warm summer&#039;s evenin&#039;[/tab]";
        let decoded = decode_html_entities(input);
        let expanded = expand_ug_tab_blocks(&decoded);
        assert!(expanded.contains("G                          C               G"));
        assert!(expanded.contains("On a warm summer's evenin'"));
        assert!(!expanded.contains("[tab]"));
        assert!(!expanded.contains("[ch]"));
    }

    #[test]
    fn fixture_html_extracts_and_parses_json() {
        let html = include_str!("../../fixtures/web/ultimate_guitar_amazing_grace.html");
        let raw = extract_js_store(html).expect("js-store");
        let parsed: Value = serde_json::from_str(&raw).expect("json");
        assert_eq!(
            parsed
                .pointer("/store/page/data/tab/song_name")
                .and_then(Value::as_str),
            Some("Amazing Grace")
        );
    }

    #[test]
    fn converts_inline_ch_markers_to_chordpro() {
        let chart = ug_inline_content_to_chordpro(
            "[Verse]\n[ch]G[/ch]Amazing [ch]D[/ch]grace",
            Some("Amazing Grace"),
            Some("Traditional"),
            Some("G"),
            Some(0),
        );
        assert!(chart.contains("{title: Amazing Grace}"));
        assert!(chart.contains("[G]Amazing [D]grace"));
        assert!(!chart.contains("[ch]"));
    }

    #[test]
    fn keeps_preamble_as_pending_annotations_for_next_section() {
        let plain = ug_tab_content_to_plain(
            "Tabbed by: Emrldeyzs\nCapo 2\n\n[Intro]\n[ch]Am[/ch] x2",
            Some("Example"),
            Some("Artist"),
            Some("Am"),
        );
        assert!(plain.contains("[Intro]\nTabbed by: Emrldeyzs\nCapo 2\nAm\nRepeat 2×"));
        assert!(!plain.contains("capo: 2"));
    }

    #[test]
    fn normalizes_inline_chord_progression_with_repeat() {
        let parts = normalize_ug_plain_line("[ch]Am[/ch]   [ch]E7[/ch]   [ch]G[/ch]   [ch]D[/ch]   x2");
        assert_eq!(parts[0], "Am   E7   G   D");
        assert_eq!(parts[1], "Repeat 2×");
    }
}
