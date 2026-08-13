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

    /// Replace lyric text. Chord indices are clamped to the new length.
    pub fn set_lyrics(&mut self, text: impl Into<String>) {
        let text = text.into();
        let max = text.chars().count() as u32;
        let (mut chords, annotations) = self.split_tokens();
        for chord in &mut chords {
            if let Some(index) = chord.lyric_index.as_mut() {
                *index = (*index).min(max);
            }
        }
        self.rebuild(chords, text, annotations);
    }

    pub fn tag_chord(&mut self, chord: Chord, lyric_index: u32) {
        let lyrics = self.lyric_text();
        let max = lyrics.chars().count() as u32;
        let (mut chords, annotations) = self.split_tokens();
        chords.push(ChordToken::new(chord).at_lyric_index(lyric_index.min(max)));
        chords.sort_by_key(|token| token.lyric_index.unwrap_or(0));
        self.rebuild(chords, lyrics, annotations);
    }

    /// # Errors
    ///
    /// Unknown chord index.
    pub fn untag_chord(&mut self, index: usize) -> Result<(), String> {
        let lyrics = self.lyric_text();
        let (mut chords, annotations) = self.split_tokens();
        if index >= chords.len() {
            return Err("That chord tag was not found.".to_string());
        }
        chords.remove(index);
        self.rebuild(chords, lyrics, annotations);
        Ok(())
    }

    /// # Errors
    ///
    /// Unknown chord index.
    pub fn replace_chord(&mut self, index: usize, chord: Chord) -> Result<(), String> {
        let lyrics = self.lyric_text();
        let (mut chords, annotations) = self.split_tokens();
        let existing = chords
            .get_mut(index)
            .ok_or_else(|| "That chord tag was not found.".to_string())?;
        let lyric_index = existing.lyric_index;
        let column = existing.column;
        let mut next = ChordToken::new(chord);
        next.lyric_index = lyric_index;
        next.column = column;
        *existing = next;
        self.rebuild(chords, lyrics, annotations);
        Ok(())
    }

    /// # Errors
    ///
    /// Unknown chord index.
    pub fn set_chord_lyric_index(&mut self, index: usize, lyric_index: u32) -> Result<(), String> {
        let lyrics = self.lyric_text();
        let max = lyrics.chars().count() as u32;
        let (mut chords, annotations) = self.split_tokens();
        let existing = chords
            .get_mut(index)
            .ok_or_else(|| "That chord tag was not found.".to_string())?;
        existing.lyric_index = Some(lyric_index.min(max));
        chords.sort_by_key(|token| token.lyric_index.unwrap_or(0));
        self.rebuild(chords, lyrics, annotations);
        Ok(())
    }

    pub fn set_annotation(&mut self, text: Option<String>) {
        let lyrics = self.lyric_text();
        let (chords, _) = self.split_tokens();
        let annotations = text
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(|value| vec![AnnotationToken::new(value)])
            .unwrap_or_default();
        self.rebuild(chords, lyrics, annotations);
    }

    fn split_tokens(&self) -> (Vec<ChordToken>, Vec<AnnotationToken>) {
        let mut chords = Vec::new();
        let mut annotations = Vec::new();
        for token in &self.tokens {
            match token {
                LineToken::Chord(chord) => chords.push(chord.clone()),
                LineToken::Annotation(annotation) => annotations.push(annotation.clone()),
                LineToken::Lyric(_) => {}
            }
        }
        (chords, annotations)
    }

    fn rebuild(
        &mut self,
        chords: Vec<ChordToken>,
        lyrics: String,
        annotations: Vec<AnnotationToken>,
    ) {
        let mut tokens: Vec<LineToken> = chords.into_iter().map(LineToken::Chord).collect();
        tokens.push(LineToken::Lyric(LyricToken::new(lyrics)));
        tokens.extend(annotations.into_iter().map(LineToken::Annotation));
        self.tokens = tokens;
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

    pub fn chord_mut(&mut self) -> &mut Chord {
        &mut self.chord
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
    fn set_lyrics_clamps_chord_indices() {
        let mut line = Line::chord_over_lyrics(
            "Hello world",
            [(parse_chord("C"), 0), (parse_chord("G"), 6)],
        );
        line.set_lyrics("Hi");
        assert_eq!(line.lyric_text(), "Hi");
        let alignments = line.chord_lyric_alignments();
        assert_eq!(alignments[0].lyric_index, 0);
        assert_eq!(alignments[1].lyric_index, 2);
    }

    #[test]
    fn tag_and_untag_chords() {
        let mut line = Line::lyrics("Amazing grace");
        line.tag_chord(parse_chord("G"), 0);
        line.tag_chord(parse_chord("D"), 8);
        assert_eq!(line.chord_tokens().count(), 2);
        line.untag_chord(0).unwrap();
        assert_eq!(line.chord_tokens().count(), 1);
        assert_eq!(
            line.chord_tokens().next().unwrap().chord().source_text(),
            "D"
        );
    }

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
