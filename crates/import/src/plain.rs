//! Plain-text chord-over-lyrics → canonical [`Song`].

use tonic_domain::{
    parse_chord, AnnotationToken, ChordToken, Key, Line, LineToken, ParseStatus, Section,
    SectionLabel, Song, SongId, SongSource, Tempo, TimeSignature,
};

use crate::warning::{ImportWarning, WarningKind};
use crate::ImportResult;

pub fn import_plain_text(input: &str, id: impl Into<SongId>) -> ImportResult {
    let mut warnings = Vec::new();
    let mut title: Option<String> = None;
    let mut artist: Option<String> = None;
    let mut album: Option<String> = None;
    let mut key: Option<Key> = None;
    let mut tempo: Option<Tempo> = None;
    let mut time_signature: Option<TimeSignature> = None;
    let mut note_lines: Vec<String> = Vec::new();

    let mut sections: Vec<Section> = Vec::new();
    let mut current_label = SectionLabel::Verse { number: None };
    let mut current_lines: Vec<Line> = Vec::new();
    let mut section_explicit = false;

    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line_no = i as u32 + 1;
        let raw = lines[i];
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        if let Some((name, value)) = metadata_prefix(trimmed) {
            apply_metadata(
                name,
                value,
                line_no,
                &mut title,
                &mut artist,
                &mut album,
                &mut key,
                &mut tempo,
                &mut time_signature,
                &mut note_lines,
                &mut warnings,
            );
            i += 1;
            continue;
        }

        if let Some(label) = parse_section_header(trimmed) {
            flush_section(
                &mut sections,
                &mut current_label,
                &mut current_lines,
                section_explicit,
            );
            current_label = label;
            section_explicit = true;
            i += 1;
            continue;
        }

        if is_no_chord_mark(trimmed) {
            current_lines.push(Line::new(vec![LineToken::Annotation(
                AnnotationToken::new(trimmed),
            )]));
            i += 1;
            continue;
        }

        let next = lines.get(i + 1).copied().unwrap_or("").trim_end();
        if is_chord_line(trimmed) && looks_like_lyrics(next) {
            let chords = parse_chord_positions(raw, line_no, &mut warnings);
            current_lines.push(Line::chord_over_lyrics(next.to_string(), chords));
            i += 2;
            continue;
        }

        if is_chord_line(trimmed) {
            let chords = parse_chord_positions(raw, line_no, &mut warnings);
            let tokens = chords
                .into_iter()
                .map(|(chord, column)| {
                    LineToken::Chord(
                        ChordToken::new(chord)
                            .at_column(column)
                            .at_lyric_index(column),
                    )
                })
                .collect();
            current_lines.push(Line::new(tokens));
            i += 1;
            continue;
        }

        current_lines.push(Line::lyrics(raw.trim_end()));
        i += 1;
    }

    flush_section(
        &mut sections,
        &mut current_label,
        &mut current_lines,
        section_explicit,
    );

    if sections.is_empty() {
        warnings.push(ImportWarning::new(
            WarningKind::MalformedInput,
            "No lyric or chord lines were found.",
            None,
        ));
    }

    let mut builder = Song::builder(id, title.unwrap_or_else(|| "Untitled".to_string()))
        .source(SongSource::plain_text(input))
        .sections(sections);

    if let Some(artist) = artist {
        builder = builder.artist(artist);
    }
    if let Some(album) = album {
        builder = builder.album(album);
    }
    if let Some(key) = key {
        builder = builder.original_key(key).performance_key(key);
    }
    if let Some(tempo) = tempo {
        builder = builder.tempo(tempo);
    }
    if let Some(time_signature) = time_signature {
        builder = builder.time_signature(time_signature);
    }
    if !note_lines.is_empty() {
        builder = builder.notes(note_lines.join("\n"));
    }

    ImportResult {
        song: builder.build(),
        warnings,
    }
}

fn metadata_prefix(line: &str) -> Option<(&str, &str)> {
    let (name, value) = line.split_once(':')?;
    let name = name.trim();
    if name.contains(' ') || name.is_empty() {
        return None;
    }
    let key = name.to_ascii_lowercase();
    matches!(
        key.as_str(),
        "title" | "artist" | "composer" | "album" | "key" | "capo" | "tempo" | "time" | "bpm"
    )
    .then_some((name, value.trim()))
}

#[allow(clippy::too_many_arguments)]
fn apply_metadata(
    name: &str,
    value: &str,
    line_no: u32,
    title: &mut Option<String>,
    artist: &mut Option<String>,
    album: &mut Option<String>,
    key: &mut Option<Key>,
    tempo: &mut Option<Tempo>,
    time_signature: &mut Option<TimeSignature>,
    note_lines: &mut Vec<String>,
    warnings: &mut Vec<ImportWarning>,
) {
    match name.to_ascii_lowercase().as_str() {
        "title" if title.is_none() && !value.is_empty() => *title = Some(value.to_string()),
        "artist" | "composer" if artist.is_none() && !value.is_empty() => {
            *artist = Some(value.to_string());
        }
        "album" if album.is_none() && !value.is_empty() => *album = Some(value.to_string()),
        "key" => {
            if let Some(parsed) = Key::parse(value) {
                *key = Some(parsed);
            } else if !value.is_empty() {
                warnings.push(ImportWarning::new(
                    WarningKind::MalformedInput,
                    format!("Could not parse key '{value}'."),
                    Some(line_no),
                ));
            }
        }
        "tempo" | "bpm" => {
            let bpm = value
                .split(|c: char| !c.is_ascii_digit())
                .rfind(|p| !p.is_empty())
                .and_then(|p| p.parse().ok())
                .and_then(Tempo::new);
            if let Some(parsed) = bpm {
                *tempo = Some(parsed);
            } else if !value.is_empty() {
                warnings.push(ImportWarning::new(
                    WarningKind::MalformedInput,
                    format!("Could not parse tempo '{value}'."),
                    Some(line_no),
                ));
            }
        }
        "time" => {
            if let Some((n, d)) = value.split_once('/') {
                if let (Ok(n), Ok(d)) = (n.trim().parse(), d.trim().parse()) {
                    if let Some(parsed) = TimeSignature::new(n, d) {
                        *time_signature = Some(parsed);
                        return;
                    }
                }
            }
            warnings.push(ImportWarning::new(
                WarningKind::MalformedInput,
                format!("Could not parse time signature '{value}'."),
                Some(line_no),
            ));
        }
        "capo" => note_lines.push(format!("Capo: {value}")),
        _ => {}
    }
}

fn parse_section_header(line: &str) -> Option<SectionLabel> {
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
        "tag" | "breakdown" => Some(SectionLabel::Custom {
            name: stripped.to_string(),
        }),
        _ => None,
    }
}

fn is_no_chord_mark(line: &str) -> bool {
    matches!(
        line.to_ascii_lowercase().as_str(),
        "n.c." | "nc" | "n/c" | "%"
    )
}

fn is_chord_line(line: &str) -> bool {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return false;
    }
    let chordish = tokens
        .iter()
        .filter(|token| {
            parse_chord(token).status() == ParseStatus::FullyRecognized || is_no_chord_mark(token)
        })
        .count();
    chordish * 2 > tokens.len()
}

fn looks_like_lyrics(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || parse_section_header(trimmed).is_some() || is_chord_line(trimmed) {
        return false;
    }
    trimmed.chars().any(|c| c.is_lowercase()) || trimmed.chars().count() > 12
}

fn parse_chord_positions(
    line: &str,
    line_no: u32,
    warnings: &mut Vec<ImportWarning>,
) -> Vec<(tonic_domain::Chord, u32)> {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        let start = i as u32;
        let mut j = i + 1;
        while j < chars.len() && !chars[j].is_whitespace() {
            j += 1;
        }
        let token: String = chars[i..j].iter().collect();
        if is_no_chord_mark(&token) {
            i = j;
            continue;
        }
        let chord = parse_chord(&token);
        match chord.status() {
            ParseStatus::FullyRecognized => {}
            ParseStatus::PartiallyRecognized => warnings.push(ImportWarning::new(
                WarningKind::PartialChord,
                format!("Partially recognized chord '{token}'."),
                Some(line_no),
            )),
            ParseStatus::Unrecognized => warnings.push(ImportWarning::new(
                WarningKind::UnrecognizedChord,
                format!("Unrecognized chord '{token}' was preserved."),
                Some(line_no),
            )),
        }
        out.push((chord, start));
        i = j;
    }
    out
}

fn flush_section(
    sections: &mut Vec<Section>,
    label: &mut SectionLabel,
    lines: &mut Vec<Line>,
    explicit: bool,
) {
    if lines.is_empty() && !explicit {
        return;
    }
    sections.push(Section::new(label.clone(), std::mem::take(lines)));
}
