//! Display DTO for the song viewer. Derived from the canonical [`Song`].

use serde::Serialize;
use tonic_domain::{Line, LineToken, ParseStatus, Song, SourceFormat};
use tonic_import::{ImportWarning, WarningKind, UNRECOGNIZED_CONTENT_MESSAGE};

/// Session snapshot returned to the UI after import or transpose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSessionView {
    pub song: SongView,
    pub warnings: Vec<WarningView>,
    pub summary_message: Option<String>,
    pub semitone_offset: i32,
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
    pub tempo_bpm: Option<u16>,
    pub time_signature: Option<String>,
    pub notes: Option<String>,
    pub source_format: String,
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
    /// Performance-key spelling to show.
    pub symbol: String,
    /// Written (authoritative) symbol from the song document.
    pub written: String,
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
    pub fn from_parts(song: &Song, warnings: &[ImportWarning], semitone_offset: i32) -> Self {
        Self {
            song: SongView::from_song(song),
            warnings: warnings.iter().map(WarningView::from).collect(),
            summary_message: (!warnings.is_empty())
                .then_some(UNRECOGNIZED_CONTENT_MESSAGE.to_string()),
            semitone_offset,
        }
    }
}

impl SongView {
    #[must_use]
    pub fn from_song(song: &Song) -> Self {
        Self {
            id: song.id().as_str().to_string(),
            title: song.title().to_string(),
            artist: song.artist().map(str::to_string),
            album: song.album().map(str::to_string),
            original_key: song.original_key().map(|key| key.symbol()),
            performance_key: song.performance_key().map(|key| key.symbol()),
            tempo_bpm: song.tempo().map(|tempo| tempo.bpm()),
            time_signature: song.time_signature().map(|ts| ts.symbol()),
            notes: song.notes().map(str::to_string),
            source_format: source_format_name(song.source().format()),
            sections: song
                .sections()
                .iter()
                .map(|section| SectionView {
                    label: section.label().display_name(),
                    lines: section
                        .lines()
                        .iter()
                        .map(|line| line_view(song, line))
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

fn line_view(song: &Song, line: &Line) -> LineView {
    let chords = line
        .chord_lyric_alignments()
        .into_iter()
        .map(|alignment| {
            let display = song.display_chord(&alignment.chord);
            ChordView {
                symbol: display.symbol(),
                written: alignment.chord.source_text().to_string(),
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

fn source_format_name(format: &SourceFormat) -> String {
    match format {
        SourceFormat::ChordPro => "chordPro".to_string(),
        SourceFormat::PlainText => "plainText".to_string(),
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
        parse_chord, ChordToken, Key, LineToken, LyricToken, Section, SectionLabel, SongId,
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

        let view = SongView::from_song(&song);
        assert_eq!(view.sections[0].lines[0].chords[0].symbol, "D");
        assert_eq!(view.sections[0].lines[0].chords[0].written, "C");
        assert_eq!(view.sections[0].lines[0].chords[1].symbol, "A");
        assert_eq!(view.sections[0].lines[0].lyrics, "Hi there");

        song.set_performance_key(Some(Key::parse("C").unwrap()));
        let reset = SongView::from_song(&song);
        assert_eq!(reset.sections[0].lines[0].chords[0].symbol, "C");
    }
}
