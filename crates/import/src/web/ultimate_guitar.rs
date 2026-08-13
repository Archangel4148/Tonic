//! Ultimate Guitar chord-tab adapter.
//!
//! UG embeds page data in `<div class="js-store" data-content="…JSON…">`.
//! Chord bodies use `[ch]Am[/ch]` markers. When wrapped in `[tab]` blocks, chords
//! sit on a spaced line above lyrics (plain-text layout). Otherwise chords are
//! inline on the lyric line (ChordPro layout).

use serde_json::Value;
use tonic_domain::{parse_chord, Key, ParseStatus, SectionLabel, SongId, SongSource};

use crate::chordpro::import_chordpro;
use crate::plain::import_plain_text;
use crate::section::{
    is_chart_annotation, is_inline_capo_line, is_repeat_marker, parse_capo_fret,
    split_leading_section_header,
};
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
        let plain = ug_tab_content_to_plain(&body, song_name, artist_name, tonality);
        import_plain_text(&plain, id)
    } else {
        let chart = ug_inline_content_to_chordpro(&body, song_name, artist_name, tonality);
        import_chordpro(&chart, id)
    };
    if result.capo_fret.is_none() {
        result.capo_fret = capo_fret_from_meta(capo);
    }

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
    // UG (and JSON-in-HTML) often double-encodes (`&amp;rsquo;` → `&rsquo;` → `'`).
    let mut out = input.to_string();
    for _ in 0..4 {
        let next = html_escape::decode_html_entities(&out).into_owned();
        if next == out {
            break;
        }
        out = next;
    }
    out
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
    let raw_lines: Vec<&str> = expanded.lines().collect();
    let appendix_from = ug_appendix_line_index(&raw_lines);
    let mut seen_section = false;
    let mut preamble: Vec<String> = Vec::new();

    for (idx, line) in raw_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if appendix_from.is_some_and(|start| idx >= start) {
            emit_plain_note(&mut out, line);
            continue;
        }
        if let Some((label, rest)) = split_leading_section_header(trimmed) {
            if !seen_section {
                flush_preamble_notes(&mut out, &mut preamble);
                seen_section = true;
            }
            out.push_str(&section_label_line(&label));
            out.push('\n');
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
        if !seen_section {
            if looks_like_ug_chart_line(line) {
                flush_preamble_notes(&mut out, &mut preamble);
                seen_section = true;
            } else {
                for part in normalize_ug_plain_line(line.trim_end()) {
                    let text = part.trim();
                    if !text.is_empty() {
                        preamble.push(text.to_string());
                    }
                }
                continue;
            }
        }
        if is_chart_annotation(trimmed) {
            out.push_str(trimmed);
            out.push('\n');
            continue;
        }
        for part in normalize_ug_plain_line(line.trim_end()) {
            out.push_str(&part);
            out.push('\n');
        }
    }
    flush_preamble_notes(&mut out, &mut preamble);
    out
}

fn flush_preamble_notes(out: &mut String, preamble: &mut Vec<String>) {
    for note in preamble.drain(..) {
        // Plain-text `note:` metadata becomes song.notes (not a default Verse).
        out.push_str("note: ");
        out.push_str(&note);
        out.push('\n');
    }
}

fn emit_plain_note(out: &mut String, line: &str) {
    let text = appendix_plain_text(line);
    if !text.is_empty() {
        out.push_str("note: ");
        out.push_str(&text);
        out.push('\n');
    }
}

fn ug_appendix_line_index(lines: &[&str]) -> Option<usize> {
    let first_chart = lines.iter().position(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty()
            && (split_leading_section_header(trimmed).is_some() || looks_like_ug_chart_line(line))
    })?;
    let mut start = None;
    for (i, line) in lines.iter().enumerate().skip(first_chart + 1) {
        if line.trim().is_empty() {
            continue;
        }
        if looks_like_ug_appendix_start(line) {
            start = Some(i);
            break;
        }
    }
    let mut idx = start?;
    while idx > first_chart {
        let prev = lines[idx - 1];
        if prev.trim().is_empty() || looks_like_appendix_heading(prev) {
            idx -= 1;
            continue;
        }
        break;
    }
    if idx <= first_chart {
        start
    } else {
        Some(idx)
    }
}

fn looks_like_appendix_heading(line: &str) -> bool {
    let plain = appendix_plain_text(line);
    looks_like_alternates_header(&plain)
        || looks_like_open_key_variant(&plain)
        || is_inline_capo_line(&plain)
}

/// Transcriber appendix after the sung chart: footnotes, capo maps, `Am = F#m`.
fn looks_like_ug_appendix_start(line: &str) -> bool {
    let plain = appendix_plain_text(line);
    looks_like_ug_footnote(&plain)
        || looks_like_alternates_header(&plain)
        || looks_like_chord_equivalence(&plain)
        || looks_like_open_key_variant(&plain)
        || looks_like_ug_transcriber_sig(&plain)
}

fn appendix_plain_text(line: &str) -> String {
    strip_ug_chord_markers(line)
        .chars()
        .filter(|c| *c != '[' && *c != ']')
        .collect::<String>()
        .trim()
        .to_string()
}

fn looks_like_ug_footnote(line: &str) -> bool {
    let trimmed = line.trim();
    let stars = trimmed.bytes().take_while(|&b| b == b'*').count();
    if stars == 0 {
        return false;
    }
    let after_stars = &trimmed[stars..];
    if !after_stars.starts_with(|c: char| c.is_whitespace()) {
        return false;
    }
    let rest = after_stars.trim();
    !rest.is_empty() && (looks_like_alternates_header(rest) || rest.len() >= 8)
}

fn looks_like_alternates_header(line: &str) -> bool {
    let lower = line
        .trim()
        .trim_end_matches(':')
        .trim()
        .to_ascii_lowercase();
    lower == "alternate"
        || lower == "alternates"
        || lower.starts_with("alternate chord")
        || lower.starts_with("alternate capo")
}

fn looks_like_open_key_variant(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    lower.starts_with("open")
        && (lower.contains("original key") || lower.contains("not in the original"))
}

fn looks_like_ug_transcriber_sig(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    lower.strip_prefix("set").is_some_and(|digits| {
        !digits.is_empty() && digits.len() <= 4 && digits.chars().all(|c| c.is_ascii_digit())
    })
}

fn looks_like_chord_equivalence(line: &str) -> bool {
    let trimmed = line.trim();
    let Some((left, right)) = trimmed.split_once('=') else {
        return false;
    };
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() || right.is_empty() || left.split_whitespace().count() != 1 {
        return false;
    }
    chord_like_token(left)
        && right.split_whitespace().any(|token| {
            chord_like_token(token.trim_matches(|c: char| matches!(c, '(' | ')' | ',' | ';' | '*')))
        })
}

fn chord_like_token(token: &str) -> bool {
    let cleaned = token
        .trim()
        .trim_matches(|c: char| matches!(c, '(' | ')' | ',' | ';' | '*'));
    if cleaned.is_empty() {
        return false;
    }
    parse_chord(cleaned).status() == ParseStatus::FullyRecognized
}

fn looks_like_ug_chart_line(line: &str) -> bool {
    let stripped = strip_ug_chord_markers(line);
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return false;
    }
    split_leading_section_header(trimmed).is_some() || crate::plain::is_chord_line(trimmed)
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

fn capo_fret_from_meta(capo: Option<u64>) -> Option<u8> {
    let fret = u8::try_from(capo?).ok()?;
    parse_capo_fret(&fret.to_string())
}

fn ug_inline_content_to_chordpro(
    content: &str,
    title: Option<&str>,
    artist: Option<&str>,
    key: Option<&str>,
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
    out.push('\n');

    let with_chords = replace_ug_chords(content);
    let without_tab = strip_tag_pairs(&with_chords, "tab");
    let normalized = without_tab
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<br>", "\n");

    let raw_lines: Vec<&str> = normalized.lines().collect();
    let appendix_from = ug_appendix_line_index(&raw_lines);
    let mut seen_section = false;
    let mut preamble: Vec<String> = Vec::new();
    for (idx, line) in raw_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if appendix_from.is_some_and(|start| idx >= start) {
            let text = appendix_plain_text(line);
            if !text.is_empty() {
                out.push_str(&format!("{{notes: {text}}}\n"));
            }
            continue;
        }
        if split_leading_section_header(trimmed).is_some()
            || trimmed.starts_with('{')
                && (trimmed.to_ascii_lowercase().contains("start_of_")
                    || trimmed.to_ascii_lowercase().contains("sov")
                    || trimmed.to_ascii_lowercase().contains("soc"))
        {
            if !seen_section {
                for note in preamble.drain(..) {
                    out.push_str(&format!("{{notes: {note}}}\n"));
                }
                seen_section = true;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if !seen_section {
            if looks_like_ug_chart_line(line) {
                for note in preamble.drain(..) {
                    out.push_str(&format!("{{notes: {note}}}\n"));
                }
                seen_section = true;
            } else {
                preamble.push(strip_ug_chord_markers(trimmed));
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    for note in preamble.drain(..) {
        out.push_str(&format!("{{notes: {note}}}\n"));
    }
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
        assert_eq!(
            decode_html_entities("summer&#039;s evenin&#39;"),
            "summer's evenin'"
        );
        assert_eq!(decode_html_entities("&amp;amp;"), "&");
    }

    #[test]
    fn decodes_named_typography_entities() {
        assert_eq!(
            decode_html_entities("I&rsquo;ll find someone like you"),
            "I’ll find someone like you"
        );
        assert_eq!(
            decode_html_entities("&ldquo;hello&rdquo; &ndash; &mdash; &hellip;"),
            "“hello” – — …"
        );
        assert_eq!(
            decode_html_entities("Tom &amp; Jerry &amp;rsquo;s"),
            "Tom & Jerry ’s"
        );
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
        );
        assert!(chart.contains("{title: Amazing Grace}"));
        assert!(chart.contains("[G]Amazing [D]grace"));
        assert!(!chart.contains("[ch]"));
    }

    #[test]
    fn preamble_before_first_section_becomes_notes() {
        let plain = ug_tab_content_to_plain(
            "Capo 5 for the original studio version  https://example.com/a\nCapo 6 for the official video  https://example.com/b\n\n[Verse 1]\n[ch]G[/ch]  [ch]Em[/ch]\nI heard there was a secret chord",
            Some("Hallelujah"),
            Some("Jeff Buckley"),
            Some("C"),
        );
        assert!(
            plain.contains("note: Capo 5 for the original studio version  https://example.com/a")
        );
        assert!(plain.contains("note: Capo 6 for the official video  https://example.com/b"));
        assert!(plain.contains("[Verse 1]"));
        let verse_idx = plain.find("[Verse 1]").unwrap();
        let note_idx = plain.find("note: Capo 5").unwrap();
        assert!(note_idx < verse_idx);
        assert!(!plain[..verse_idx].contains("I heard"));
    }

    #[test]
    fn keeps_preamble_credits_as_notes_not_first_section() {
        let plain = ug_tab_content_to_plain(
            "Tabbed by: Emrldeyzs\nCapo 2\n\n[Intro]\n[ch]Am[/ch] x2",
            Some("Example"),
            Some("Artist"),
            Some("Am"),
        );
        assert!(plain.contains("note: Tabbed by: Emrldeyzs"));
        assert!(plain.contains("note: Capo 2"));
        assert!(plain.contains("[Intro]\nAm\nRepeat 2×"));
        assert!(!plain.contains("capo: 2"));
    }

    #[test]
    fn unlabeled_chart_starts_at_first_chord_line() {
        let plain = ug_tab_content_to_plain(
            "Fly Me To The Moon chords\nhttps://example.com/wiki\n\n[tab][ch]Am[/ch]          [ch]Dm7[/ch]\nFly me to the moon[/tab]",
            Some("Fly Me To The Moon"),
            Some("Frank Sinatra"),
            Some("C"),
        );
        assert!(plain.contains("note: Fly Me To The Moon chords"));
        assert!(plain.contains("note: https://example.com/wiki"));
        assert!(plain.contains("Am          Dm7"));
        assert!(plain.contains("Fly me to the moon"));
        let chord_idx = plain.find("Am          Dm7").unwrap();
        let note_idx = plain.find("note: Fly Me To The Moon chords").unwrap();
        assert!(note_idx < chord_idx);
    }

    #[test]
    fn trailing_transcriber_appendix_becomes_notes() {
        let plain = ug_tab_content_to_plain(
            "[tab]   [ch]Dm7[/ch]          [ch]G[/ch] [ch]G7[/ch]   [ch]C[/ch]\nIn other words  I love you![/tab]\n\n*    It has been suggested that Am7, Dm7 and Fmaj7 could be used as an option\n**   It has been suggested that instead of ending with Cmaj7 try ending with C then Cmaj7\n\n***  Alternates:\n\nCapo III\n\n[ch]Am[/ch]    = [ch]F#m[/ch]\n[ch]Dm7[/ch]   = [ch]Bm7[/ch] ****\n[ch]Cmaj7[/ch] = [ch]G[/ch] (or [ch]Gmaj7[/ch])\n\nOpen (These chords are not in the original key)\n\n[ch]Am[/ch]    = [ch]Bm[/ch]\n\nSet8",
            Some("Fly Me To The Moon"),
            Some("Frank Sinatra"),
            Some("C"),
        );
        assert!(plain.contains("In other words  I love you!"));
        assert!(plain.contains(
            "note: *    It has been suggested that Am7, Dm7 and Fmaj7 could be used as an option"
        ));
        assert!(plain.contains("note: Am    = F#m"));
        assert!(plain.contains("note: Capo III"));
        assert!(plain.contains("note: Open (These chords are not in the original key)"));
        assert!(plain.contains("note: Set8"));
        let lyric_idx = plain.find("I love you!").unwrap();
        for line in plain[lyric_idx..].lines().skip(1) {
            if line.contains("It has been suggested")
                || line.contains("Am    = F#m")
                || line.contains("Capo III")
                || line.contains("Set8")
            {
                assert!(
                    line.trim_start().starts_with("note:"),
                    "appendix line should be a note: {line}"
                );
            }
        }
    }

    #[test]
    fn capo_heading_before_chord_map_is_pulled_into_notes() {
        let plain = ug_tab_content_to_plain(
            "[tab][ch]C[/ch]\nHello[/tab]\n\nCapo V\n[ch]Am[/ch]    = [ch]Em[/ch]\n[ch]G7[/ch]    = [ch]D7[/ch]",
            Some("Example"),
            Some("Artist"),
            Some("C"),
        );
        assert!(plain.contains("Hello"));
        assert!(plain.contains("note: Capo V"));
        assert!(plain.contains("note: Am    = Em"));
        assert!(!plain.lines().any(|line| line.trim() == "Capo V"));
    }

    #[test]
    fn starred_adlib_lyric_is_not_an_appendix() {
        let plain = ug_tab_content_to_plain(
            "[tab][ch]C[/ch]\n*whoa*[/tab]\n[tab][ch]G[/ch]\nmore lyrics[/tab]",
            Some("Example"),
            Some("Artist"),
            Some("C"),
        );
        assert!(plain.contains("*whoa*"));
        assert!(!plain.contains("note: *whoa*"));
        assert!(plain.contains("more lyrics"));
    }

    #[test]
    fn parenthesized_chord_groups_stay_on_the_chord_line() {
        let plain = ug_tab_content_to_plain(
            "[Intro]\n( [ch]Am[/ch] [ch]Dm[/ch] [ch]G[/ch] ) [ch]C[/ch]",
            Some("Example"),
            Some("Artist"),
            Some("C"),
        );
        assert!(plain.contains("[Intro]"));
        assert!(
            plain.contains("( Am Dm G ) C")
                || plain.contains("(Am Dm G) C")
                || plain.contains("( Am Dm G )")
        );
    }

    #[test]
    fn normalizes_inline_chord_progression_with_repeat() {
        let parts =
            normalize_ug_plain_line("[ch]Am[/ch]   [ch]E7[/ch]   [ch]G[/ch]   [ch]D[/ch]   x2");
        assert_eq!(parts[0], "Am   E7   G   D");
        assert_eq!(parts[1], "Repeat 2×");
    }

    #[test]
    fn listed_ug_capo_is_captured_as_import_fret() {
        let html = r#"<div class="js-store" data-content="{&quot;store&quot;:{&quot;page&quot;:{&quot;data&quot;:{&quot;tab&quot;:{&quot;song_name&quot;:&quot;Demo&quot;,&quot;artist_name&quot;:&quot;A&quot;},&quot;tab_view&quot;:{&quot;meta&quot;:{&quot;capo&quot;:2,&quot;tonality&quot;:&quot;G&quot;},&quot;wiki_tab&quot;:{&quot;content&quot;:&quot;[tab][ch]G[/ch]\nHello[/tab]&quot;}}}}}}"></div>"#;
        let result = parse_ultimate_guitar_html(
            "https://tabs.ultimate-guitar.com/tab/a/demo-chords-1",
            html,
            SongId::new("song-1"),
        )
        .expect("parse");
        assert_eq!(result.capo_fret, Some(2));
        assert_eq!(
            result
                .song
                .original_key()
                .map(|key| key.symbol())
                .as_deref(),
            Some("G")
        );
    }
}
