//! Song editor draft DTOs and canonical-model mutations.
//!
//! The UI never parses chords. Rust owns the draft until Save or Cancel.

use serde::{Deserialize, Serialize};
use tonic_domain::{
    parse_chord, Key, Line, ParseStatus, Section, SectionLabel, Song, SongId, SongSource, Tempo,
    TimeSignature,
};
use tonic_import::{ImportWarning, WarningKind, UNRECOGNIZED_CONTENT_MESSAGE};

use crate::library;
use crate::SongSessionView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSaveResult {
    pub session: SongSessionView,
    pub editor: EditorSessionView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSessionView {
    pub song_id: String,
    pub dirty: bool,
    pub is_new: bool,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub original_key: Option<String>,
    pub tempo_bpm: Option<u16>,
    pub time_signature: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub warnings: Vec<crate::WarningView>,
    pub summary_message: Option<String>,
    pub sections: Vec<EditorSectionView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSectionView {
    pub label: String,
    pub kind: String,
    pub number: Option<u16>,
    pub custom_name: Option<String>,
    pub lines: Vec<EditorLineView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorLineView {
    pub lyrics: String,
    pub chords: Vec<EditorChordView>,
    pub annotation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorChordView {
    pub symbol: String,
    pub lyric_index: u32,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorMetaUpdate {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub original_key: Option<String>,
    pub tempo_bpm: Option<u16>,
    pub time_signature: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionLabelInput {
    pub kind: String,
    pub number: Option<u16>,
    pub custom_name: Option<String>,
}

pub(crate) struct EditorSession {
    pub draft: Song,
    pub baseline: Option<Song>,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub is_new: bool,
    pub dirty: bool,
    pub warnings: Vec<ImportWarning>,
}

impl EditorSession {
    pub(crate) fn new_song(id: SongId) -> Self {
        Self {
            draft: blank_song(id),
            baseline: None,
            tags: Vec::new(),
            favorite: false,
            is_new: true,
            dirty: true,
            warnings: Vec::new(),
        }
    }

    pub(crate) fn from_library(song: Song, tags: Vec<String>, favorite: bool) -> Self {
        Self {
            draft: song.clone(),
            baseline: Some(song),
            tags,
            favorite,
            is_new: false,
            dirty: false,
            warnings: Vec::new(),
        }
    }
}

#[must_use]
pub(crate) fn editor_view(session: &EditorSession) -> EditorSessionView {
    let warnings: Vec<crate::WarningView> = session.warnings.iter().map(Into::into).collect();
    let summary =
        (!session.warnings.is_empty()).then_some(UNRECOGNIZED_CONTENT_MESSAGE.to_string());
    EditorSessionView {
        song_id: session.draft.id().as_str().to_string(),
        dirty: session.dirty,
        is_new: session.is_new,
        title: session.draft.title().to_string(),
        artist: session.draft.artist().map(str::to_string),
        album: session.draft.album().map(str::to_string),
        original_key: session.draft.original_key().map(|key| key.symbol()),
        tempo_bpm: session.draft.tempo().map(|tempo| tempo.bpm()),
        time_signature: session
            .draft
            .time_signature()
            .map(|signature| signature.symbol()),
        notes: session.draft.notes().map(str::to_string),
        tags: session.tags.clone(),
        warnings,
        summary_message: summary,
        sections: session
            .draft
            .sections()
            .iter()
            .map(|section| EditorSectionView {
                label: section.label().display_name(),
                kind: section.label().kind_key().to_string(),
                number: section.label().number(),
                custom_name: section.label().custom_name().map(str::to_string),
                lines: section
                    .lines()
                    .iter()
                    .map(|line| EditorLineView {
                        lyrics: line.lyric_text(),
                        chords: line
                            .chord_lyric_alignments()
                            .into_iter()
                            .map(|alignment| EditorChordView {
                                symbol: alignment.chord.source_text().to_string(),
                                lyric_index: alignment.lyric_index,
                                status: parse_status_name(alignment.chord.status()).to_string(),
                            })
                            .collect(),
                        annotation: line
                            .tokens()
                            .iter()
                            .filter_map(|token| match token {
                                tonic_domain::LineToken::Annotation(annotation) => {
                                    Some(annotation.text().to_string())
                                }
                                _ => None,
                            })
                            .next(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

pub(crate) fn blank_song(id: SongId) -> Song {
    Song::builder(id, "Untitled")
        .source(SongSource::manual())
        .section(Section::new(
            SectionLabel::Verse { number: None },
            vec![Line::lyrics("")],
        ))
        .build()
}

pub(crate) fn apply_meta(song: &mut Song, update: &EditorMetaUpdate) -> Result<(), String> {
    let title = update.title.trim();
    if title.is_empty() {
        return Err("Title cannot be empty.".to_string());
    }
    song.set_title(title);
    song.set_artist(library::blank_to_none(update.artist.clone()));
    song.set_album(library::blank_to_none(update.album.clone()));
    song.set_notes(library::blank_to_none(update.notes.clone()));

    let key_text = update
        .original_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(symbol) = key_text {
        let key = Key::parse(symbol).ok_or_else(|| format!("Unknown key '{symbol}'."))?;
        let previous = song.original_key();
        song.set_original_key(Some(key));
        if song.performance_key().is_none() || song.performance_key() == previous {
            song.set_performance_key(Some(key));
        }
    } else {
        song.set_original_key(None);
    }

    match update.tempo_bpm {
        Some(bpm) => {
            let tempo = Tempo::new(bpm).ok_or_else(|| format!("Invalid tempo '{bpm}'."))?;
            song.set_tempo(Some(tempo));
        }
        None => song.set_tempo(None),
    }

    match update
        .time_signature
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(symbol) => song.set_time_signature(Some(parse_time_signature(symbol)?)),
        None => song.set_time_signature(None),
    }
    Ok(())
}

pub(crate) fn parse_time_signature(symbol: &str) -> Result<TimeSignature, String> {
    let (numerator, denominator) = symbol
        .split_once('/')
        .ok_or_else(|| format!("Time signature '{symbol}' should look like 4/4."))?;
    let numerator = numerator
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid time signature '{symbol}'."))?;
    let denominator = denominator
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid time signature '{symbol}'."))?;
    TimeSignature::new(numerator, denominator)
        .ok_or_else(|| format!("Invalid time signature '{symbol}'."))
}

pub(crate) fn line_mut(song: &mut Song, section: usize, line: usize) -> Result<&mut Line, String> {
    song.sections_mut()
        .get_mut(section)
        .ok_or_else(|| "That section was not found.".to_string())?
        .lines_mut()
        .get_mut(line)
        .ok_or_else(|| "That line was not found.".to_string())
}

pub(crate) fn parse_label(input: &SectionLabelInput) -> Result<SectionLabel, String> {
    SectionLabel::parse(&input.kind, input.number, input.custom_name.as_deref())
}

pub(crate) fn refresh_chord_warnings(session: &mut EditorSession) {
    session.warnings = warnings_from_song(&session.draft);
}

pub(crate) fn warnings_from_song(song: &Song) -> Vec<ImportWarning> {
    let mut warnings = Vec::new();
    for (section_index, section) in song.sections().iter().enumerate() {
        for (line_index, line) in section.lines().iter().enumerate() {
            for chord in line.chord_tokens() {
                let location =
                    format!("{} line {}", section.label().display_name(), line_index + 1);
                match chord.chord().status() {
                    ParseStatus::Unrecognized => warnings.push(ImportWarning::new(
                        WarningKind::UnrecognizedChord,
                        format!(
                            "{location}: unrecognized chord '{}'.",
                            chord.chord().source_text()
                        ),
                        Some((section_index + 1) as u32),
                    )),
                    ParseStatus::PartiallyRecognized => warnings.push(ImportWarning::new(
                        WarningKind::PartialChord,
                        format!(
                            "{location}: partial chord '{}'.",
                            chord.chord().source_text()
                        ),
                        Some((section_index + 1) as u32),
                    )),
                    ParseStatus::FullyRecognized => {}
                }
            }
        }
    }
    warnings
}

pub(crate) fn parse_chord_symbol(symbol: &str) -> Result<tonic_domain::Chord, String> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        return Err("Chord symbol cannot be empty.".to_string());
    }
    Ok(parse_chord(trimmed))
}

fn parse_status_name(status: ParseStatus) -> &'static str {
    match status {
        ParseStatus::FullyRecognized => "fullyRecognized",
        ParseStatus::PartiallyRecognized => "partiallyRecognized",
        ParseStatus::Unrecognized => "unrecognized",
    }
}
