//! Display DTO for the song viewer. Derived from the canonical [`Song`].

use serde::Serialize;
use tonic_domain::{
    transpose_musicxml_text, transpose_to_key, Key, Line, LineToken, ParseStatus, Song,
    SourceFormat, Spelling,
};
use tonic_import::{
    ImportWarning, WarningKind, UNRECOGNIZED_CONTENT_MESSAGE, UNSUPPORTED_MUSICXML_MESSAGE,
};
use tonic_persist::TransposeMode;

use crate::infer_key_from_content;
use crate::setlist::SetlistContextView;

/// Session snapshot returned to the UI after import or transpose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSessionView {
    pub song: SongView,
    pub warnings: Vec<WarningView>,
    pub summary_message: Option<String>,
    pub semitone_offset: i32,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub setlist: Option<SetlistContextView>,
    /// How the performance key is realized: rewrite chords, or keep shapes + capo.
    pub transpose_mode: TransposeMode,
    /// Capo fret when using capo transpose (library or setlist).
    pub capo_fret: Option<u8>,
    /// Fingered key when a capo is in use (written shapes).
    pub played_key: Option<String>,
    /// Derived MusicXML for OSMD. `None` when the song has no score.
    pub sheet_music_xml: Option<String>,
}

/// Render-ready song. Display chord symbols are already transposed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongView {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub original_key: Option<String>,
    pub performance_key: Option<String>,
    /// Effective key for transpose controls: performance, original, or inferred from content.
    pub display_key: Option<String>,
    pub tempo_bpm: Option<u16>,
    pub time_signature: Option<String>,
    pub notes: Option<String>,
    pub source_format: String,
    pub has_score: bool,
    pub sections: Vec<SectionView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionView {
    pub label: String,
    pub lines: Vec<LineView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineView {
    pub lyrics: String,
    pub chords: Vec<ChordView>,
    pub annotations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChordView {
    /// Performance-key spelling, or written shape when using capo transpose.
    pub symbol: String,
    /// Written (authoritative) symbol from the song document.
    pub written: String,
    /// Concert/sounding spelling (same as `symbol` unless capo mode).
    pub sounding: String,
    pub lyric_index: u32,
    pub column: Option<u32>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WarningView {
    pub kind: String,
    pub message: String,
    pub line: Option<u32>,
}

impl SongSessionView {
    #[must_use]
    pub fn from_parts(
        song: &Song,
        warnings: &[ImportWarning],
        semitone_offset: i32,
        favorite: bool,
        tags: Vec<String>,
        setlist: Option<SetlistContextView>,
        transpose_mode: TransposeMode,
        capo_fret: Option<u8>,
        played_key: Option<String>,
        shapes_key: Option<Key>,
    ) -> Self {
        Self {
            song: SongView::from_song(song, transpose_mode, shapes_key),
            warnings: warnings.iter().map(WarningView::from).collect(),
            summary_message: summary_from_warnings(warnings),
            semitone_offset,
            favorite,
            tags,
            setlist,
            transpose_mode,
            capo_fret,
            played_key,
            sheet_music_xml: sheet_music_xml(song, semitone_offset),
        }
    }
}

impl SongView {
    #[must_use]
    pub fn from_song(song: &Song, transpose_mode: TransposeMode, shapes_key: Option<Key>) -> Self {
        Self {
            id: song.id().as_str().to_string(),
            title: song.title().to_string(),
            artist: song.artist().map(str::to_string),
            album: song.album().map(str::to_string),
            original_key: song.original_key().map(|key| key.symbol()),
            performance_key: song.performance_key().map(|key| key.symbol()),
            display_key: display_key(song),
            tempo_bpm: song.tempo().map(|tempo| tempo.bpm()),
            time_signature: song.time_signature().map(|ts| ts.symbol()),
            notes: song.notes().map(str::to_string),
            source_format: source_format_name(song.source().format()),
            has_score: song.score().is_some_and(|score| !score.parts.is_empty()),
            sections: song
                .sections()
                .iter()
                .map(|section| SectionView {
                    label: section.label().display_name(),
                    lines: section
                        .lines()
                        .iter()
                        .map(|line| line_view(song, line, transpose_mode, shapes_key))
                        .collect(),
                })
                .collect(),
        }
    }
}

impl From<&ImportWarning> for WarningView {
    fn from(warning: &ImportWarning) -> Self {
        Self {
            kind: warning_kind_name(warning.kind).to_string(),
            message: warning.message.clone(),
            line: warning.line,
        }
    }
}

fn display_key(song: &Song) -> Option<String> {
    song.performance_key()
        .or(song.original_key())
        .or_else(|| infer_key_from_content(song))
        .map(|key| key.symbol())
}

fn line_view(
    song: &Song,
    line: &Line,
    transpose_mode: TransposeMode,
    shapes_key: Option<Key>,
) -> LineView {
    let chords = line
        .chord_lyric_alignments()
        .into_iter()
        .map(|alignment| {
            let sounding = song.display_chord(&alignment.chord);
            let keep_shapes = matches!(transpose_mode, TransposeMode::Capo);
            let symbol = if keep_shapes {
                match (song.original_key(), shapes_key) {
                    (Some(from), Some(to)) if from != to => {
                        transpose_to_key(&alignment.chord, from, to).symbol()
                    }
                    _ => alignment.chord.source_text().to_string(),
                }
            } else {
                sounding.symbol()
            };
            ChordView {
                symbol,
                written: alignment.chord.source_text().to_string(),
                sounding: sounding.symbol(),
                lyric_index: alignment.lyric_index,
                column: alignment.column,
                status: parse_status_name(alignment.chord.status()).to_string(),
            }
        })
        .collect();
    let annotations = line
        .tokens()
        .iter()
        .filter_map(|token| match token {
            LineToken::Annotation(annotation) => Some(annotation.text().to_string()),
            _ => None,
        })
        .collect();
    LineView {
        lyrics: line.lyric_text(),
        chords,
        annotations,
    }
}

fn sheet_music_xml(song: &Song, steps: i32) -> Option<String> {
    let spelling = match song.performance_key().or(song.original_key()) {
        Some(key) => Spelling::InKey(key),
        None => Spelling::PreserveAccidentalFamily,
    };
    if matches!(song.source().format(), SourceFormat::MusicXml) {
        if let Some(original) = song.source().original_content() {
            if original.contains("<score-partwise") || original.contains("<score-timewise") {
                return Some(transpose_musicxml_text(original, steps, spelling));
            }
        }
    }
    let score = song.score()?;
    if score.parts.is_empty() {
        return None;
    }
    let display = if steps == 0 {
        score.clone()
    } else {
        score.transpose_semitones(steps, spelling)
    };
    Some(display.to_musicxml())
}

pub(crate) fn summary_from_warnings(warnings: &[ImportWarning]) -> Option<String> {
    if warnings.is_empty() {
        None
    } else if warnings
        .iter()
        .all(|warning| warning.kind == WarningKind::UnsupportedFeature)
    {
        Some(UNSUPPORTED_MUSICXML_MESSAGE.to_string())
    } else {
        Some(UNRECOGNIZED_CONTENT_MESSAGE.to_string())
    }
}

fn source_format_name(format: &SourceFormat) -> String {
    match format {
        SourceFormat::ChordPro => "chordPro".to_string(),
        SourceFormat::PlainText => "plainText".to_string(),
        SourceFormat::MusicXml => "musicXml".to_string(),
        SourceFormat::Web => "web".to_string(),
        SourceFormat::Manual => "manual".to_string(),
        SourceFormat::Other(name) => name.clone(),
    }
}

fn parse_status_name(status: ParseStatus) -> &'static str {
    match status {
        ParseStatus::FullyRecognized => "fullyRecognized",
        ParseStatus::PartiallyRecognized => "partiallyRecognized",
        ParseStatus::Unrecognized => "unrecognized",
    }
}

fn warning_kind_name(kind: WarningKind) -> &'static str {
    match kind {
        WarningKind::UnrecognizedChord => "unrecognizedChord",
        WarningKind::PartialChord => "partialChord",
        WarningKind::UnrecognizedDirective => "unrecognizedDirective",
        WarningKind::MalformedInput => "malformedInput",
        WarningKind::AmbiguousLayout => "ambiguousLayout",
        WarningKind::SkippedContent => "skippedContent",
        WarningKind::UnsupportedFeature => "unsupportedFeature",
    }
}

/// Keys offered in the transpose dropdown. Matches [`Key::from_pitch_class`] spellings.
#[must_use]
pub fn performance_key_choices() -> Vec<String> {
    [
        "C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B", "Cm", "C#m", "Dm", "Ebm",
        "Em", "Fm", "F#m", "Gm", "G#m", "Am", "Bbm", "Bm",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic_domain::{
        parse_chord, ChordToken, Key, LineToken, LyricToken, Section, SectionLabel, Song, SongId,
    };

    #[test]
    fn view_uses_display_chords_without_rewriting_written() {
        let line = tonic_domain::Line::new(vec![
            LineToken::Chord(ChordToken::new(parse_chord("C"))),
            LineToken::Lyric(LyricToken::new("Hi ")),
            LineToken::Chord(ChordToken::new(parse_chord("G"))),
            LineToken::Lyric(LyricToken::new("there")),
        ]);
        let mut song = tonic_domain::Song::builder(SongId::new("v"), "Demo")
            .original_key(Key::parse("C").unwrap())
            .performance_key(Key::parse("D").unwrap())
            .section(Section::new(
                SectionLabel::Verse { number: None },
                vec![line],
            ))
            .build();

        let view = SongView::from_song(&song, TransposeMode::Chords, None);
        assert_eq!(view.sections[0].lines[0].chords[0].symbol, "D");
        assert_eq!(view.sections[0].lines[0].chords[0].written, "C");
        assert_eq!(view.sections[0].lines[0].chords[1].symbol, "A");
        assert_eq!(view.sections[0].lines[0].lyrics, "Hi there");

        song.set_performance_key(Some(Key::parse("C").unwrap()));
        let reset = SongView::from_song(&song, TransposeMode::Chords, None);
        assert_eq!(reset.sections[0].lines[0].chords[0].symbol, "C");

        song.set_performance_key(Some(Key::parse("D").unwrap()));
        let capo = SongView::from_song(&song, TransposeMode::Capo, None);
        assert_eq!(capo.sections[0].lines[0].chords[0].symbol, "C");
        assert_eq!(capo.sections[0].lines[0].chords[0].written, "C");
        assert_eq!(capo.sections[0].lines[0].chords[0].sounding, "D");
        assert_eq!(capo.sections[0].lines[0].chords[1].symbol, "G");
        assert_eq!(capo.sections[0].lines[0].chords[1].sounding, "A");

        let g_shapes = SongView::from_song(
            &song,
            TransposeMode::Capo,
            Some(Key::parse("G").unwrap()),
        );
        assert_eq!(g_shapes.sections[0].lines[0].chords[0].symbol, "G");
        assert_eq!(g_shapes.sections[0].lines[0].chords[0].sounding, "D");
        assert_eq!(g_shapes.sections[0].lines[0].chords[1].symbol, "D");
        assert_eq!(g_shapes.sections[0].lines[0].chords[1].sounding, "A");
    }

    #[test]
    fn display_key_prefers_performance_then_original_then_inference() {
        let line = tonic_domain::Line::new(vec![
            LineToken::Chord(ChordToken::new(parse_chord("G"))),
            LineToken::Lyric(LyricToken::new("Hi")),
        ]);
        let section = Section::new(SectionLabel::Verse { number: None }, vec![line]);

        let with_performance = Song::builder(SongId::new("perf"), "Demo")
            .original_key(Key::parse("C").unwrap())
            .performance_key(Key::parse("D").unwrap())
            .section(section.clone())
            .build();
        assert_eq!(
            SongView::from_song(&with_performance, TransposeMode::Chords, None)
                .display_key
                .as_deref(),
            Some("D")
        );

        let original_only = Song::builder(SongId::new("orig"), "Demo")
            .original_key(Key::parse("C").unwrap())
            .section(section.clone())
            .build();
        assert_eq!(
            SongView::from_song(&original_only, TransposeMode::Chords, None)
                .display_key
                .as_deref(),
            Some("C")
        );

        let inferred = Song::builder(SongId::new("inf"), "Demo")
            .section(section)
            .build();
        assert_eq!(
            SongView::from_song(&inferred, TransposeMode::Chords, None)
                .display_key
                .as_deref(),
            Some("G")
        );
    }
}
