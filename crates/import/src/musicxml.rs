//! MusicXML and compressed MXL import into [`Score`] plus song metadata.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};

use roxmltree::{Document, Node};
use tonic_domain::{
    parse_chord, Accidental, ChordToken, Clef, Key, Letter, Line, LineToken, LyricToken, Measure,
    MeasureAttributes, MeasureEvent, Note, Score, ScoreHarmony, ScoreNote, ScorePart, ScorePitch,
    ScoreRest, Section, SectionLabel, Song, SongId, SongSource, Tempo, TimeSignature,
};

use crate::warning::{ImportWarning, WarningKind};
use crate::ImportResult;

/// Import MusicXML text (partwise or timewise).
#[must_use]
pub fn import_musicxml(input: &str, id: impl Into<SongId>) -> ImportResult {
    parse_musicxml(input, id.into())
}

/// Import UTF-8 MusicXML or a `.mxl` zip payload.
#[must_use]
pub fn import_musicxml_bytes(
    bytes: &[u8],
    file_name: Option<&str>,
    id: impl Into<SongId>,
) -> ImportResult {
    match decode_score_text(bytes, file_name) {
        Ok(xml) => import_musicxml(&xml, id),
        Err(message) => ImportResult::new(
            Song::builder(id, "Untitled score")
                .source(SongSource::music_xml(
                    String::from_utf8_lossy(bytes).into_owned(),
                ))
                .build(),
            vec![ImportWarning::new(
                WarningKind::MalformedInput,
                message,
                None,
            )],
        ),
    }
}

pub(crate) fn looks_like_musicxml(input: &str) -> bool {
    let trimmed = input.trim_start();
    trimmed.contains("<score-partwise")
        || trimmed.contains("<score-timewise")
        || trimmed.contains(":score-partwise")
        || trimmed.contains(":score-timewise")
}

pub(crate) fn is_mxl_bytes(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK")
}

fn decode_score_text(bytes: &[u8], file_name: Option<&str>) -> Result<String, String> {
    let name = file_name.unwrap_or("").to_ascii_lowercase();
    if is_mxl_bytes(bytes) || name.ends_with(".mxl") {
        return extract_mxl(bytes);
    }
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn extract_mxl(bytes: &[u8]) -> Result<String, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("Could not read MXL archive: {error}"))?;
    let container_xml = match archive.by_name("META-INF/container.xml") {
        Ok(mut container) => {
            let mut text = String::new();
            container
                .read_to_string(&mut text)
                .map_err(|error| format!("Could not read MXL container: {error}"))?;
            Some(text)
        }
        Err(_) => None,
    };
    if let Some(text) = container_xml.as_deref() {
        if let Some(path) = rootfile_path(text) {
            let mut score = archive
                .by_name(&path)
                .map_err(|error| format!("MXL is missing '{path}': {error}"))?;
            let mut xml = String::new();
            score
                .read_to_string(&mut xml)
                .map_err(|error| format!("Could not read MXL score: {error}"))?;
            return Ok(xml);
        }
    }
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| format!("Could not read MXL entry: {error}"))?;
        let name = file.name().to_string();
        let lower = name.to_ascii_lowercase();
        if !(lower.ends_with(".xml") || lower.ends_with(".musicxml")) {
            continue;
        }
        if lower.contains("container.xml") || lower.contains("meta-inf") {
            continue;
        }
        let mut xml = String::new();
        file.read_to_string(&mut xml)
            .map_err(|error| format!("Could not read '{name}': {error}"))?;
        return Ok(xml);
    }
    Err("MXL archive did not contain a MusicXML score.".to_string())
}

fn rootfile_path(container_xml: &str) -> Option<String> {
    let doc = Document::parse_with_options(
        container_xml,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        },
    )
    .ok()?;
    doc.descendants()
        .find(|node| node.is_element() && local_name(*node) == "rootfile")
        .and_then(|node| {
            node.attribute("full-path")
                .or_else(|| node.attribute("fullPath"))
                .map(str::to_string)
        })
}

fn parse_musicxml(input: &str, id: SongId) -> ImportResult {
    let mut warnings = Vec::new();
    let document = match Document::parse_with_options(
        input,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        },
    ) {
        Ok(document) => document,
        Err(error) => {
            warnings.push(ImportWarning::new(
                WarningKind::MalformedInput,
                format!("MusicXML could not be parsed: {error}"),
                None,
            ));
            return ImportResult::new(
                Song::builder(id, "Untitled score")
                    .source(SongSource::music_xml(input.to_string()))
                    .build(),
                warnings,
            );
        }
    };
    let root = document.root_element();
    let root_name = local_name(root);
    if root_name != "score-partwise" && root_name != "score-timewise" {
        warnings.push(ImportWarning::new(
            WarningKind::MalformedInput,
            format!("Expected a MusicXML score, found <{root_name}>."),
            None,
        ));
        return ImportResult::new(
            Song::builder(id, "Untitled score")
                .source(SongSource::music_xml(input.to_string()))
                .build(),
            warnings,
        );
    }

    let mut unsupported = HashSet::new();
    if root_name == "score-timewise" {
        warnings.push(ImportWarning::new(
            WarningKind::UnsupportedFeature,
            "Timewise MusicXML was converted into a partwise score.".to_string(),
            None,
        ));
    }

    let title = work_title(root).unwrap_or_else(|| "Untitled score".to_string());
    let artist = creator(root);
    let (parts, part_warnings) = parse_parts(root, &mut unsupported);
    warnings.extend(part_warnings);
    for tag in unsupported {
        warnings.push(ImportWarning::new(
            WarningKind::UnsupportedFeature,
            format!("Unsupported MusicXML feature was skipped: {tag}."),
            None,
        ));
    }

    if parts.is_empty() {
        warnings.push(ImportWarning::new(
            WarningKind::MalformedInput,
            "No playable parts were found in this MusicXML file.".to_string(),
            None,
        ));
    }

    let score = Score {
        work_title: Some(title.clone()),
        parts,
    };
    let key = first_key(&score);
    let time = first_time(&score);
    let tempo = first_tempo(root);
    let sections = chart_from_score(&score);
    let mut builder = Song::builder(id, title)
        .source(SongSource::music_xml(input.to_string()))
        .score(score)
        .sections(sections);
    if let Some(artist) = artist {
        builder = builder.artist(artist);
    }
    if let Some(key) = key {
        builder = builder.original_key(key).performance_key(key);
    }
    if let Some(time) = time {
        builder = builder.time_signature(time);
    }
    if let Some(tempo) = tempo {
        builder = builder.tempo(tempo);
    }
    ImportResult::new(builder.build(), warnings)
}

fn parse_parts(
    root: Node<'_, '_>,
    unsupported: &mut HashSet<String>,
) -> (Vec<ScorePart>, Vec<ImportWarning>) {
    let mut warnings = Vec::new();
    let names = part_names(root);
    let parts = if local_name(root) == "score-partwise" {
        children_named(root, "part")
            .map(|part_node| {
                let id = part_node.attribute("id").unwrap_or("P1").to_string();
                let name = names.get(&id).cloned().unwrap_or_else(|| id.clone());
                let measures = children_named(part_node, "measure")
                    .map(|measure| parse_measure(measure, unsupported, &mut warnings))
                    .collect();
                ScorePart { id, name, measures }
            })
            .collect()
    } else {
        parse_timewise_parts(root, &names, unsupported, &mut warnings)
    };
    (parts, warnings)
}

fn parse_timewise_parts(
    root: Node<'_, '_>,
    names: &HashMap<String, String>,
    unsupported: &mut HashSet<String>,
    warnings: &mut Vec<ImportWarning>,
) -> Vec<ScorePart> {
    let mut by_id: Vec<(String, Vec<Measure>)> =
        names.keys().cloned().map(|id| (id, Vec::new())).collect();
    if by_id.is_empty() {
        by_id.push(("P1".into(), Vec::new()));
    }
    for measure_node in children_named(root, "measure") {
        let number = measure_node
            .attribute("number")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1);
        for part_node in children_named(measure_node, "part") {
            let id = part_node.attribute("id").unwrap_or("P1").to_string();
            let parsed = parse_measure_body(part_node, number, unsupported, warnings);
            if let Some((_, measures)) = by_id.iter_mut().find(|(part_id, _)| *part_id == id) {
                measures.push(parsed);
            } else {
                by_id.push((id, vec![parsed]));
            }
        }
    }
    by_id
        .into_iter()
        .map(|(id, measures)| ScorePart {
            name: names.get(&id).cloned().unwrap_or_else(|| id.clone()),
            id,
            measures,
        })
        .collect()
}

fn parse_measure(
    measure: Node<'_, '_>,
    unsupported: &mut HashSet<String>,
    warnings: &mut Vec<ImportWarning>,
) -> Measure {
    let number = measure
        .attribute("number")
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    parse_measure_body(measure, number, unsupported, warnings)
}

fn parse_measure_body(
    events_parent: Node<'_, '_>,
    number: u32,
    unsupported: &mut HashSet<String>,
    warnings: &mut Vec<ImportWarning>,
) -> Measure {
    let mut attributes = None;
    let mut events = Vec::new();
    for child in events_parent.children().filter(Node::is_element) {
        match local_name(child) {
            "attributes" => attributes = Some(parse_attributes(child)),
            "note" => events.push(parse_note(child, unsupported)),
            "harmony" => events.push(parse_harmony(child, warnings)),
            "backup" => {
                if let Some(duration) =
                    child_text(child, "duration").and_then(|text| text.parse().ok())
                {
                    events.push(MeasureEvent::Backup(duration));
                }
            }
            "forward" => {
                if let Some(duration) =
                    child_text(child, "duration").and_then(|text| text.parse().ok())
                {
                    events.push(MeasureEvent::Forward(duration));
                }
            }
            "direction" => note_unsupported(unsupported, "direction"),
            "figured-bass" => note_unsupported(unsupported, "figured-bass"),
            "barline" | "print" | "sound" | "listening" => {}
            other => note_unsupported(unsupported, other),
        }
    }
    Measure {
        number,
        attributes,
        events,
    }
}

fn parse_attributes(node: Node<'_, '_>) -> MeasureAttributes {
    let divisions = child_text(node, "divisions").and_then(|text| text.parse().ok());
    let key = children_named(node, "key").next().and_then(parse_key);
    let time = children_named(node, "time").next().and_then(parse_time);
    let clef = children_named(node, "clef").next().map(parse_clef);
    MeasureAttributes {
        divisions,
        key,
        time,
        clef,
    }
}

fn parse_key(node: Node<'_, '_>) -> Option<Key> {
    let fifths = child_text(node, "fifths")?.parse::<i32>().ok()?;
    let mode = child_text(node, "mode").unwrap_or_default();
    let minor = mode.eq_ignore_ascii_case("minor");
    Key::from_fifths(fifths, minor)
}

fn parse_time(node: Node<'_, '_>) -> Option<TimeSignature> {
    let beats = child_text(node, "beats")?.parse().ok()?;
    let beat_type = child_text(node, "beat-type")?.parse().ok()?;
    TimeSignature::new(beats, beat_type)
}

fn parse_clef(node: Node<'_, '_>) -> Clef {
    let sign = child_text(node, "sign").unwrap_or_else(|| "G".into());
    let line = child_text(node, "line").and_then(|text| text.parse().ok());
    Clef::from_musicxml(&sign, line)
}

fn parse_note(node: Node<'_, '_>, unsupported: &mut HashSet<String>) -> MeasureEvent {
    if node.descendants().any(|child| {
        child.is_element()
            && matches!(
                local_name(child),
                "tuplet" | "ornaments" | "technical" | "glissando" | "slide"
            )
    }) {
        note_unsupported(unsupported, "advanced notation");
    }
    let duration = child_text(node, "duration")
        .and_then(|text| text.parse().ok())
        .unwrap_or(1);
    let voice = child_text(node, "voice")
        .and_then(|text| text.parse().ok())
        .unwrap_or(1);
    let staff = child_text(node, "staff")
        .and_then(|text| text.parse().ok())
        .unwrap_or(1);
    let note_type = child_text(node, "type");
    let lyric = children_named(node, "lyric")
        .next()
        .and_then(|lyric| child_text(lyric, "text"));
    let chord = node
        .children()
        .any(|child| child.is_element() && local_name(child) == "chord");
    if node
        .children()
        .any(|child| child.is_element() && local_name(child) == "rest")
        || node
            .children()
            .any(|child| child.is_element() && local_name(child) == "unpitched")
    {
        if node
            .children()
            .any(|child| child.is_element() && local_name(child) == "unpitched")
        {
            note_unsupported(unsupported, "unpitched");
        }
        return MeasureEvent::Rest(ScoreRest {
            duration,
            voice,
            staff,
            note_type,
        });
    }
    let pitch = children_named(node, "pitch")
        .next()
        .and_then(parse_pitch)
        .unwrap_or_else(|| ScorePitch::new(Note::natural(Letter::C), 4));
    MeasureEvent::Note(ScoreNote {
        pitch,
        duration,
        voice,
        staff,
        chord,
        lyric,
        note_type,
    })
}

fn parse_pitch(node: Node<'_, '_>) -> Option<ScorePitch> {
    let step = child_text(node, "step")?;
    let letter = Letter::from_char(step.chars().next()?)?;
    let alter = child_text(node, "alter")
        .and_then(|text| text.parse::<i32>().ok())
        .unwrap_or(0);
    let accidental = Accidental::from_semitones(alter).unwrap_or(Accidental::Natural);
    let octave = child_text(node, "octave")
        .and_then(|text| text.parse().ok())
        .unwrap_or(4);
    Some(ScorePitch::new(Note::new(letter, accidental), octave))
}

fn parse_harmony(node: Node<'_, '_>, warnings: &mut Vec<ImportWarning>) -> MeasureEvent {
    let root_node = children_named(node, "root").next();
    let step = root_node
        .and_then(|root| child_text(root, "root-step"))
        .unwrap_or_else(|| "C".into());
    let alter = root_node
        .and_then(|root| child_text(root, "root-alter"))
        .and_then(|text| text.parse::<i32>().ok())
        .unwrap_or(0);
    let accidental = Accidental::from_semitones(alter).unwrap_or(Accidental::Natural);
    let letter = Letter::from_char(step.chars().next().unwrap_or('C')).unwrap_or(Letter::C);
    let root = Note::new(letter, accidental);
    let kind_node = children_named(node, "kind").next();
    let kind = kind_node
        .and_then(|kind| {
            kind.text()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_string)
                .or_else(|| kind.attribute("text").map(str::to_string))
        })
        .unwrap_or_else(|| "major".into());
    let suffix = harmony_suffix(&kind);
    if suffix.is_none() && !kind.eq_ignore_ascii_case("major") && !kind.eq_ignore_ascii_case("none")
    {
        warnings.push(ImportWarning::new(
            WarningKind::UnsupportedFeature,
            format!("Unsupported harmony kind '{kind}' was preserved as a chord symbol."),
            None,
        ));
    }
    let symbol = format!("{}{}", root.symbol(), suffix.unwrap_or(""));
    let chord = parse_chord(&symbol);
    MeasureEvent::Harmony(ScoreHarmony {
        symbol,
        chord: chord.root().is_some().then_some(chord),
    })
}

fn harmony_suffix(kind: &str) -> Option<&'static str> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "major" | "major-sixth" | "none" => Some(""),
        "minor" | "minor-sixth" => Some("m"),
        "augmented" => Some("aug"),
        "diminished" => Some("dim"),
        "dominant" | "dominant-seventh" => Some("7"),
        "major-seventh" => Some("maj7"),
        "minor-seventh" => Some("m7"),
        "half-diminished" => Some("m7b5"),
        "diminished-seventh" => Some("dim7"),
        "suspended-fourth" => Some("sus4"),
        "suspended-second" => Some("sus2"),
        _ => None,
    }
}

fn chart_from_score(score: &Score) -> Vec<Section> {
    let mut lines = Vec::new();
    for part in &score.parts {
        for measure in &part.measures {
            let mut tokens = Vec::new();
            let mut lyrics = String::new();
            for event in &measure.events {
                match event {
                    MeasureEvent::Harmony(harmony) => {
                        let chord = harmony
                            .chord
                            .clone()
                            .unwrap_or_else(|| parse_chord(&harmony.symbol));
                        tokens.push(LineToken::Chord(
                            ChordToken::new(chord).at_lyric_index(lyrics.chars().count() as u32),
                        ));
                    }
                    MeasureEvent::Note(note) => {
                        if let Some(lyric) = &note.lyric {
                            if !lyrics.is_empty() {
                                lyrics.push(' ');
                            }
                            lyrics.push_str(lyric);
                        }
                    }
                    _ => {}
                }
            }
            if !lyrics.is_empty() {
                tokens.push(LineToken::Lyric(LyricToken::new(lyrics)));
            }
            if !tokens.is_empty() {
                lines.push(Line::new(tokens));
            }
        }
    }
    if lines.is_empty() {
        Vec::new()
    } else {
        vec![Section::new(
            SectionLabel::Custom {
                name: "Score".into(),
            },
            lines,
        )]
    }
}

fn work_title(root: Node<'_, '_>) -> Option<String> {
    children_named(root, "work")
        .next()
        .and_then(|work| child_text(work, "work-title"))
        .or_else(|| child_text(root, "movement-title"))
        .or_else(|| {
            children_named(root, "credit")
                .flat_map(|credit| children_named(credit, "credit-words"))
                .find_map(|words| words.text().map(str::trim).map(str::to_string))
                .filter(|text| !text.is_empty())
        })
}

fn creator(root: Node<'_, '_>) -> Option<String> {
    let identification = children_named(root, "identification").next()?;
    let mut composer = None;
    let mut lyricist = None;
    let mut other = None;
    for node in children_named(identification, "creator") {
        let value = node.text()?.trim();
        if value.is_empty() {
            continue;
        }
        match node.attribute("type").unwrap_or_default() {
            "composer" if composer.is_none() => composer = Some(value.to_string()),
            "lyricist" if lyricist.is_none() => lyricist = Some(value.to_string()),
            _ if other.is_none() => other = Some(value.to_string()),
            _ => {}
        }
    }
    composer.or(lyricist).or(other)
}

fn part_names(root: Node<'_, '_>) -> HashMap<String, String> {
    let mut names = HashMap::new();
    if let Some(list) = children_named(root, "part-list").next() {
        for part in children_named(list, "score-part") {
            let id = part.attribute("id").unwrap_or("P1").to_string();
            let name = child_text(part, "part-name").unwrap_or_else(|| id.clone());
            names.insert(id, name);
        }
    }
    names
}

fn first_key(score: &Score) -> Option<Key> {
    score.parts.iter().find_map(|part| {
        part.measures.iter().find_map(|measure| {
            measure
                .attributes
                .as_ref()
                .and_then(|attributes| attributes.key)
        })
    })
}

fn first_time(score: &Score) -> Option<TimeSignature> {
    score.parts.iter().find_map(|part| {
        part.measures.iter().find_map(|measure| {
            measure
                .attributes
                .as_ref()
                .and_then(|attributes| attributes.time)
        })
    })
}

fn first_tempo(root: Node<'_, '_>) -> Option<Tempo> {
    root.descendants().find_map(|node| {
        if !node.is_element() {
            return None;
        }
        if local_name(node) == "sound" {
            if let Some(tempo) = node.attribute("tempo").and_then(|value| value.parse().ok()) {
                return Tempo::new(tempo);
            }
        }
        if local_name(node) == "per-minute" {
            if let Some(text) = node.text() {
                let digits: String = text.chars().filter(|ch| ch.is_ascii_digit()).collect();
                return digits.parse().ok().and_then(Tempo::new);
            }
        }
        None
    })
}

fn note_unsupported(set: &mut HashSet<String>, tag: &str) {
    set.insert(tag.to_string());
}

fn children_named<'a, 'input>(
    node: Node<'a, 'input>,
    name: &'a str,
) -> impl Iterator<Item = Node<'a, 'input>> + 'a {
    node.children()
        .filter(move |child| child.is_element() && local_name(*child) == name)
}

fn child_text(node: Node<'_, '_>, name: &str) -> Option<String> {
    children_named(node, name)
        .next()?
        .text()
        .map(str::trim)
        .map(str::to_string)
        .filter(|text| !text.is_empty())
}

fn local_name<'a>(node: Node<'a, '_>) -> &'a str {
    node.tag_name().name()
}
