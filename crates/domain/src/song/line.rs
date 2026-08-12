//! Line tokens: chords, lyrics, and annotations with recoverable positions.

use serde::{Deserialize, Serialize};

use crate::chord::Chord;

/// One visual/musical line inside a section.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    #[serde(default)]
    tokens: Vec<LineToken>,
}

/// Ordered tokens that make up a line.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LineToken {
    Chord(ChordToken),
    Lyric(LyricToken),
    Annotation(AnnotationToken),
}

/// A chord placed relative to lyrics on the same line.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChordToken {
    chord: Chord,
    /// Character index into this line's concatenated lyric text, if known.
    #[serde(default)]
    lyric_index: Option<u32>,
    /// Monospaced column for chord-over-lyric layouts, if known.
    #[serde(default)]
    column: Option<u32>,
}

/// Lyric text. Adjacent lyric tokens concatenate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricToken {
    text: String,
}

/// Comment or performance annotation attached to a line.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationToken {
    text: String,
    #[serde(default)]
    lyric_index: Option<u32>,
}

/// Recovered chord-to-lyric placement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChordAlignment {
    pub chord: Chord,
    pub lyric_index: u32,
    pub column: Option<u32>,
}

impl Line {
    #[must_use]
    pub fn new(tokens: Vec<LineToken>) -> Self {
        Self { tokens }
    }

    #[must_use]
    pub fn lyrics(text: impl Into<String>) -> Self {
        Self {
            tokens: vec![LineToken::Lyric(LyricToken::new(text))],
        }
    }

    /// Chord-over-lyric line: one lyric string plus chords at character columns.
    #[must_use]
    pub fn chord_over_lyrics(
        lyrics: impl Into<String>,
        chords: impl IntoIterator<Item = (Chord, u32)>,
    ) -> Self {
        let lyrics = lyrics.into();
        let mut tokens = Vec::new();
        for (chord, column) in chords {
            let lyric_index = column.min(lyrics.chars().count() as u32);
            tokens.push(LineToken::Chord(
                ChordToken::new(chord)
                    .at_column(column)
                    .at_lyric_index(lyric_index),
            ));
        }
        tokens.push(LineToken::Lyric(LyricToken::new(lyrics)));
        Self { tokens }
    }

    #[must_use]
    pub fn tokens(&self) -> &[LineToken] {
        &self.tokens
    }

    #[must_use]
    pub fn lyric_text(&self) -> String {
        self.tokens
            .iter()
            .filter_map(|token| match token {
                LineToken::Lyric(lyric) => Some(lyric.text.as_str()),
                _ => None,
            })
            .collect()
    }

    pub fn chord_tokens(&self) -> impl Iterator<Item = &ChordToken> {
        self.tokens.iter().filter_map(|token| match token {
            LineToken::Chord(chord) => Some(chord),
            _ => None,
        })
    }

    /// Recover chord placements. Missing `lyric_index` is inferred from inline order.
    #[must_use]
    pub fn chord_lyric_alignments(&self) -> Vec<ChordAlignment> {
        let mut lyric_cursor = 0_u32;
        let mut alignments = Vec::new();
        for token in &self.tokens {
            match token {
                LineToken::Lyric(lyric) => {
                    lyric_cursor += lyric.text.chars().count() as u32;
                }
                LineToken::Chord(chord) => {
                    alignments.push(ChordAlignment {
                        chord: chord.chord.clone(),
                        lyric_index: chord.lyric_index.unwrap_or(lyric_cursor),
                        column: chord.column,
                    });
                }
                LineToken::Annotation(_) => {}
            }
        }
        alignments
    }
}

impl ChordToken {
    #[must_use]
    pub fn new(chord: Chord) -> Self {
        Self {
            chord,
            lyric_index: None,
            column: None,
        }
    }

    #[must_use]
    pub fn at_lyric_index(mut self, lyric_index: u32) -> Self {
        self.lyric_index = Some(lyric_index);
        self
    }

    #[must_use]
    pub fn at_column(mut self, column: u32) -> Self {
        self.column = Some(column);
        self
    }

    #[must_use]
    pub fn chord(&self) -> &Chord {
        &self.chord
    }

    #[must_use]
    pub fn lyric_index(&self) -> Option<u32> {
        self.lyric_index
    }

    #[must_use]
    pub fn column(&self) -> Option<u32> {
        self.column
    }
}

impl LyricToken {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl AnnotationToken {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            lyric_index: None,
        }
    }

    #[must_use]
    pub fn at_lyric_index(mut self, lyric_index: u32) -> Self {
        self.lyric_index = Some(lyric_index);
        self
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn lyric_index(&self) -> Option<u32> {
        self.lyric_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_chord;

    #[test]
    fn infers_inline_chord_positions() {
        let line = Line::new(vec![
            LineToken::Chord(ChordToken::new(parse_chord("G"))),
            LineToken::Lyric(LyricToken::new("Amazing grace, how ")),
            LineToken::Chord(ChordToken::new(parse_chord("D"))),
            LineToken::Lyric(LyricToken::new("sweet the sound")),
        ]);

        let alignments = line.chord_lyric_alignments();
        assert_eq!(line.lyric_text(), "Amazing grace, how sweet the sound");
        assert_eq!(alignments.len(), 2);
        assert_eq!(alignments[0].chord.symbol(), "G");
        assert_eq!(alignments[0].lyric_index, 0);
        assert_eq!(alignments[1].chord.symbol(), "D");
        assert_eq!(
            alignments[1].lyric_index,
            "Amazing grace, how ".chars().count() as u32
        );
    }

    #[test]
    fn chord_over_lyrics_keeps_columns() {
        let line = Line::chord_over_lyrics(
            "Amazing grace how sweet",
            [(parse_chord("C"), 0), (parse_chord("G"), 11)],
        );
        let alignments = line.chord_lyric_alignments();
        assert_eq!(alignments[0].column, Some(0));
        assert_eq!(alignments[1].column, Some(11));
        assert_eq!(alignments[1].lyric_index, 11);
        assert_eq!(line.lyric_text(), "Amazing grace how sweet");
    }
}
