//! Canonical in-memory song document.
//!
//! Written chords and source text are authoritative. Transposed display chords
//! are derived from `original_key` → `performance_key`.

mod line;
mod meta;
mod section;
mod source;

pub use line::{AnnotationToken, ChordAlignment, ChordToken, Line, LineToken, LyricToken};
pub use meta::{Tempo, TimeSignature, Timestamp};
pub use section::{Section, SectionLabel};
pub use source::{SongSource, SourceFormat};

use serde::{Deserialize, Serialize};

use crate::chord::Chord;
use crate::key::Key;
use crate::score::Score;
use crate::transpose::transpose_to_key;

/// Stable opaque song identifier. Generation is an application concern.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SongId(String);

impl SongId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SongId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Normalized song. The renderer and editor must consume this, not raw import text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    id: SongId,
    title: String,
    #[serde(default)]
    artist: Option<String>,
    #[serde(default)]
    album: Option<String>,
    #[serde(default)]
    original_key: Option<Key>,
    #[serde(default)]
    performance_key: Option<Key>,
    #[serde(default)]
    tempo: Option<Tempo>,
    #[serde(default)]
    time_signature: Option<TimeSignature>,
    #[serde(default)]
    sections: Vec<Section>,
    #[serde(default)]
    score: Option<Score>,
    #[serde(default)]
    source: SongSource,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    created_at: Option<Timestamp>,
    #[serde(default)]
    updated_at: Option<Timestamp>,
}

impl Song {
    #[must_use]
    pub fn builder(id: impl Into<SongId>, title: impl Into<String>) -> SongBuilder {
        SongBuilder::new(id.into(), title.into())
    }

    #[must_use]
    pub fn id(&self) -> &SongId {
        &self.id
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    #[must_use]
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    #[must_use]
    pub fn original_key(&self) -> Option<Key> {
        self.original_key
    }

    #[must_use]
    pub fn performance_key(&self) -> Option<Key> {
        self.performance_key
    }

    #[must_use]
    pub fn tempo(&self) -> Option<Tempo> {
        self.tempo
    }

    #[must_use]
    pub fn time_signature(&self) -> Option<TimeSignature> {
        self.time_signature
    }

    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub fn sections_mut(&mut self) -> &mut Vec<Section> {
        &mut self.sections
    }

    #[must_use]
    pub fn score(&self) -> Option<&Score> {
        self.score.as_ref()
    }

    pub fn set_score(&mut self, score: Option<Score>) {
        self.score = score;
    }

    #[must_use]
    pub fn source(&self) -> &SongSource {
        &self.source
    }

    #[must_use]
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    #[must_use]
    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }

    #[must_use]
    pub fn updated_at(&self) -> Option<Timestamp> {
        self.updated_at
    }

    pub fn set_id(&mut self, id: impl Into<SongId>) {
        self.id = id.into();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_artist(&mut self, artist: Option<String>) {
        self.artist = artist;
    }

    pub fn set_album(&mut self, album: Option<String>) {
        self.album = album;
    }

    pub fn set_original_key(&mut self, key: Option<Key>) {
        self.original_key = key;
    }

    pub fn set_performance_key(&mut self, key: Option<Key>) {
        self.performance_key = key;
    }

    pub fn set_tempo(&mut self, tempo: Option<Tempo>) {
        self.tempo = tempo;
    }

    pub fn set_time_signature(&mut self, time_signature: Option<TimeSignature>) {
        self.time_signature = time_signature;
    }

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
    }

    pub fn set_source(&mut self, source: SongSource) {
        self.source = source;
    }

    pub fn set_created_at(&mut self, created_at: Option<Timestamp>) {
        self.created_at = created_at;
    }

    pub fn set_updated_at(&mut self, updated_at: Option<Timestamp>) {
        self.updated_at = updated_at;
    }

    /// Derived display spelling. Does not mutate written chords or source text.
    #[must_use]
    pub fn display_chord(&self, chord: &Chord) -> Chord {
        match (self.original_key, self.performance_key) {
            (Some(from), Some(to)) if from != to => transpose_to_key(chord, from, to),
            _ => chord.clone(),
        }
    }

    /// JSON interchange for the canonical model. Not durable storage.
    ///
    /// # Errors
    ///
    /// Returns any [`serde_json`] serialization error.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Inverse of [`Song::to_json`].
    ///
    /// # Errors
    ///
    /// Returns any [`serde_json`] deserialization error.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Fluent constructor for [`Song`].
#[derive(Debug)]
pub struct SongBuilder {
    id: SongId,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    original_key: Option<Key>,
    performance_key: Option<Key>,
    tempo: Option<Tempo>,
    time_signature: Option<TimeSignature>,
    sections: Vec<Section>,
    score: Option<Score>,
    source: SongSource,
    notes: Option<String>,
    created_at: Option<Timestamp>,
    updated_at: Option<Timestamp>,
}

impl SongBuilder {
    fn new(id: SongId, title: String) -> Self {
        Self {
            id,
            title,
            artist: None,
            album: None,
            original_key: None,
            performance_key: None,
            tempo: None,
            time_signature: None,
            sections: Vec::new(),
            score: None,
            source: SongSource::manual(),
            notes: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[must_use]
    pub fn artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = Some(artist.into());
        self
    }

    #[must_use]
    pub fn album(mut self, album: impl Into<String>) -> Self {
        self.album = Some(album.into());
        self
    }

    #[must_use]
    pub fn original_key(mut self, key: Key) -> Self {
        self.original_key = Some(key);
        self
    }

    #[must_use]
    pub fn performance_key(mut self, key: Key) -> Self {
        self.performance_key = Some(key);
        self
    }

    #[must_use]
    pub fn tempo(mut self, tempo: Tempo) -> Self {
        self.tempo = Some(tempo);
        self
    }

    #[must_use]
    pub fn time_signature(mut self, time_signature: TimeSignature) -> Self {
        self.time_signature = Some(time_signature);
        self
    }

    #[must_use]
    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    #[must_use]
    pub fn sections(mut self, sections: Vec<Section>) -> Self {
        self.sections = sections;
        self
    }

    #[must_use]
    pub fn score(mut self, score: Score) -> Self {
        self.score = Some(score);
        self
    }

    #[must_use]
    pub fn source(mut self, source: SongSource) -> Self {
        self.source = source;
        self
    }

    #[must_use]
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    #[must_use]
    pub fn created_at(mut self, timestamp: Timestamp) -> Self {
        self.created_at = Some(timestamp);
        self
    }

    #[must_use]
    pub fn updated_at(mut self, timestamp: Timestamp) -> Self {
        self.updated_at = Some(timestamp);
        self
    }

    #[must_use]
    pub fn build(self) -> Song {
        Song {
            id: self.id,
            title: self.title,
            artist: self.artist,
            album: self.album,
            original_key: self.original_key,
            performance_key: self.performance_key,
            tempo: self.tempo,
            time_signature: self.time_signature,
            sections: self.sections,
            score: self.score,
            source: self.source,
            notes: self.notes,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
