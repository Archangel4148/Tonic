//! ChordPro (`.cho` / `.crd`) → canonical [`Song`].

use tonic_domain::{
    parse_chord, AnnotationToken, ChordToken, Key, Line, LineToken, LyricToken, ParseStatus,
    Section, SectionLabel, Song, SongId, SongSource, Tempo, TimeSignature,
};

use crate::section::{
    extract_capo_directive, is_layout_marker, is_prose_annotation, split_leading_section_header,
};
use crate::warning::{ImportWarning, WarningKind};
use crate::ImportResult;

pub fn import_chordpro(input: &str, id: impl Into<SongId>) -> ImportResult {
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
    let mut skip_remaining = false;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx as u32 + 1;
        if skip_remaining {
            continue;
        }

        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            // ChordPro `#` comments are not part of the performed song.
            continue;
        }

        if trimmed.starts_with('{') {
            match parse_directive(trimmed) {
                None => {
                    warnings.push(ImportWarning::new(
                        WarningKind::MalformedInput,
                        format!("Malformed directive: {trimmed}"),
                        Some(line_no),
                    ));
                }
                Some((name, value, closed)) => {
                    if !closed {
                        warnings.push(ImportWarning::new(
                            WarningKind::MalformedInput,
                            format!("Unclosed directive: {trimmed}"),
                            Some(line_no),
                        ));
                    }
                    let name = normalize_name(&name);
                    if matches!(name.as_str(), "meta") {
                        let (meta_key, meta_val) = split_meta(&value);
                        apply_directive(
                            &normalize_name(meta_key),
                            meta_val,
                            line_no,
                            &mut title,
                            &mut artist,
                            &mut album,
                            &mut key,
                            &mut tempo,
                            &mut time_signature,
                            &mut note_lines,
                            &mut warnings,
                            &mut sections,
                            &mut current_label,
                            &mut current_lines,
                            &mut section_explicit,
                            &mut skip_remaining,
                        );
                    } else {
                        apply_directive(
                            &name,
                            &value,
                            line_no,
                            &mut title,
                            &mut artist,
                            &mut album,
                            &mut key,
                            &mut tempo,
                            &mut time_signature,
                            &mut note_lines,
                            &mut warnings,
                            &mut sections,
                            &mut current_label,
                            &mut current_lines,
                            &mut section_explicit,
                            &mut skip_remaining,
                        );
                    }
                }
            }
            continue;
        }

        if let Some(capo) = extract_capo_directive(trimmed) {
            current_lines.push(Line::new(vec![LineToken::Annotation(
                AnnotationToken::new(capo),
            )]));
            continue;
        }

        if is_prose_annotation(trimmed) {
            current_lines.push(Line::new(vec![LineToken::Annotation(
                AnnotationToken::new(trimmed),
            )]));
            continue;
        }

        if let Some((label, rest)) = split_leading_section_header(trimmed) {
            flush_section(
                &mut sections,
                &mut current_label,
                &mut current_lines,
                section_explicit,
            );
            current_label = label;
            section_explicit = true;
            if !rest.is_empty() {
                let (line, line_warnings) = parse_chordpro_line(rest, line_no);
                warnings.extend(line_warnings);
                current_lines.push(line);
            }
            continue;
        }

        let (line, line_warnings) = parse_chordpro_line(raw_line, line_no);
        warnings.extend(line_warnings);
        current_lines.push(line);
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
        .source(SongSource::chordpro(input))
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

fn has_song_body(title: &Option<String>, sections: &[Section], lines: &[Line]) -> bool {
    title.is_some()
        || !sections.is_empty()
        || lines.iter().any(|line| {
            line.tokens()
                .iter()
                .any(|token| matches!(token, LineToken::Chord(_) | LineToken::Lyric(_)))
        })
}

fn parse_directive(line: &str) -> Option<(String, String, bool)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    let closed = trimmed.ends_with('}');
    let inner = trimmed.trim_start_matches('{').trim_end_matches('}').trim();
    if inner.is_empty() {
        return None;
    }
    let (name, value) = inner
        .split_once(':')
        .map(|(n, v)| (n.trim(), v.trim()))
        .unwrap_or((inner, ""));
    Some((name.to_string(), value.to_string(), closed))
}

fn normalize_name(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

fn split_meta(value: &str) -> (&str, &str) {
    value
        .split_once(char::is_whitespace)
        .map(|(k, v)| (k.trim(), v.trim()))
        .unwrap_or((value, ""))
}

fn alias_name(name: &str) -> &str {
    match name {
        "t" => "title",
        "st" => "subtitle",
        "k" => "key",
        "c" | "ci" | "comment_italic" => "comment",
        "sov" => "start_of_verse",
        "eov" => "end_of_verse",
        "soc" => "start_of_chorus",
        "eoc" => "end_of_chorus",
        "sob" => "start_of_bridge",
        "eob" => "end_of_bridge",
        "sot" => "start_of_tab",
        "eot" => "end_of_tab",
        "bpm" => "tempo",
        "ns" => "new_song",
        other => other,
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_directive(
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
    sections: &mut Vec<Section>,
    current_label: &mut SectionLabel,
    current_lines: &mut Vec<Line>,
    section_explicit: &mut bool,
    skip_remaining: &mut bool,
) {
    let name = alias_name(name);
    match name {
        "title" => {
            if !value.is_empty() {
                if title.is_none() {
                    *title = Some(value.to_string());
                } else if artist.is_none() {
                    *artist = Some(value.to_string());
                }
            }
        }
        "artist" => {
            if !value.is_empty() {
                *artist = Some(value.to_string());
            }
        }
        "composer" | "subtitle" => {
            if artist.is_none() && !value.is_empty() {
                *artist = Some(
                    value
                        .trim_matches(|c: char| matches!(c, '(' | ')'))
                        .trim()
                        .to_string(),
                );
            }
        }
        "album" => {
            if album.is_none() && !value.is_empty() {
                *album = Some(value.to_string());
            }
        }
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
        "tempo" => {
            if let Some(parsed) = parse_tempo(value) {
                *tempo = Some(parsed);
            } else if !value.is_empty() {
                warnings.push(ImportWarning::new(
                    WarningKind::MalformedInput,
                    format!("Could not parse tempo '{value}'."),
                    Some(line_no),
                ));
            }
        }
        "time" | "time_signature" => {
            if let Some(parsed) = parse_time_signature(value) {
                *time_signature = Some(parsed);
            } else if !value.is_empty() {
                warnings.push(ImportWarning::new(
                    WarningKind::MalformedInput,
                    format!("Could not parse time signature '{value}'."),
                    Some(line_no),
                ));
            }
        }
        "capo" => {
            if !value.is_empty() {
                let text = format!("Capo {value}");
                note_lines.push(text.clone());
                current_lines.push(Line::new(vec![LineToken::Annotation(
                    AnnotationToken::new(text),
                )]));
            }
        }
        "comment" => {
            if !value.is_empty() {
                current_lines.push(Line::new(vec![LineToken::Annotation(
                    AnnotationToken::new(value),
                )]));
            }
        }
        "new_song" => {
            // Many real .pro dumps start with `{ns}`. Only skip if a song body
            // has already been collected.
            if has_song_body(title, sections, current_lines) {
                *skip_remaining = true;
                warnings.push(ImportWarning::new(
                    WarningKind::SkippedContent,
                    "Additional songs in this file were skipped.",
                    Some(line_no),
                ));
            }
        }
        "define" | "chord" => {}
        name if name.starts_with("start_of_") => {
            flush_section(sections, current_label, current_lines, *section_explicit);
            *current_label = section_label_from_start(name, value);
            *section_explicit = true;
        }
        name if name.starts_with("end_of_") => {
            flush_section(sections, current_label, current_lines, *section_explicit);
            *current_label = SectionLabel::Verse { number: None };
            *section_explicit = false;
        }
        "chorus" if value.is_empty() => {
            flush_section(sections, current_label, current_lines, *section_explicit);
            *current_label = SectionLabel::Chorus { number: None };
            *section_explicit = true;
        }
        _ => {
            warnings.push(ImportWarning::new(
                WarningKind::UnrecognizedDirective,
                format!("Unrecognized directive '{{{name}}}'."),
                Some(line_no),
            ));
        }
    }
}

fn section_label_from_start(name: &str, extra: &str) -> SectionLabel {
    let kind = name.trim_start_matches("start_of_");
    let extra = extra.trim();
    let number = if extra.is_empty() {
        None
    } else {
        extract_number(extra)
    };
    match kind {
        "verse" => SectionLabel::Verse { number },
        "chorus" => SectionLabel::Chorus { number },
        "prechorus" | "pre_chorus" => SectionLabel::PreChorus,
        "bridge" => SectionLabel::Bridge,
        "intro" => SectionLabel::Intro,
        "outro" => SectionLabel::Outro,
        "solo" => SectionLabel::Solo,
        "instrumental" | "tab" => SectionLabel::Instrumental,
        _ if !extra.is_empty() => SectionLabel::Custom {
            name: extra.to_string(),
        },
        other => SectionLabel::Custom {
            name: other.replace('_', " "),
        },
    }
}

fn extract_number(s: &str) -> Option<u16> {
    s.split_whitespace()
        .rev()
        .find_map(|part| part.parse().ok())
}

fn parse_tempo(value: &str) -> Option<Tempo> {
    let digits: String = value
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    let bpm = if digits.is_empty() {
        value
            .split(|c: char| !c.is_ascii_digit())
            .rfind(|p| !p.is_empty())?
            .parse()
            .ok()?
    } else {
        digits.parse().ok()?
    };
    Tempo::new(bpm)
}

fn parse_time_signature(value: &str) -> Option<TimeSignature> {
    let (n, d) = value.split_once('/')?;
    TimeSignature::new(n.trim().parse().ok()?, d.trim().parse().ok()?)
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

fn parse_chordpro_line(line: &str, line_no: u32) -> (Line, Vec<ImportWarning>) {
    let mut tokens = Vec::new();
    let mut warnings = Vec::new();
    let mut rest = line.trim_end();

    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix('[') {
            match stripped.find(']') {
                None => {
                    warnings.push(ImportWarning::new(
                        WarningKind::MalformedInput,
                        "Unclosed chord bracket; remaining text kept as lyrics.",
                        Some(line_no),
                    ));
                    tokens.push(LineToken::Lyric(LyricToken::new(rest)));
                    break;
                }
                Some(end) => {
                    let chord_text = stripped[..end].trim();
                    if is_layout_marker(chord_text) {
                        // Bar lines, N.C., empty `[]` — keep lyrics flowing.
                    } else {
                        let chord = parse_chord(chord_text);
                        match chord.status() {
                            ParseStatus::FullyRecognized => {}
                            ParseStatus::PartiallyRecognized => warnings.push(ImportWarning::new(
                                WarningKind::PartialChord,
                                format!("Partially recognized chord '{chord_text}'."),
                                Some(line_no),
                            )),
                            ParseStatus::Unrecognized => warnings.push(ImportWarning::new(
                                WarningKind::UnrecognizedChord,
                                format!("Unrecognized chord '{chord_text}' was preserved."),
                                Some(line_no),
                            )),
                        }
                        tokens.push(LineToken::Chord(ChordToken::new(chord)));
                    }
                    rest = &stripped[end + 1..];
                }
            }
        } else if let Some(idx) = rest.find('[') {
            tokens.push(LineToken::Lyric(LyricToken::new(&rest[..idx])));
            rest = &rest[idx..];
        } else {
            tokens.push(LineToken::Lyric(LyricToken::new(rest)));
            break;
        }
    }

    (Line::new(tokens), warnings)
}
