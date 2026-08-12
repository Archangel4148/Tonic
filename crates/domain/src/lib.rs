//! Music-theory and song-domain layer for Tonic.
//!
//! This crate must remain independent of UI, Tauri, and persistence.

mod capo;
mod chord;
mod key;
mod note;
mod parse;
mod pitch;
mod song;
mod transpose;

pub use capo::{concert_key, concert_pitch, played_key, played_shape, Capo, CapoError};
pub use chord::{
    AddedTone, Alteration, Chord, Extension, ParseStatus, Quality, Seventh, Suspension,
};
pub use key::{AccidentalPreference, Key, Mode};
pub use note::{Accidental, Letter, Note, Spelling};
pub use parse::parse_chord;
pub use pitch::{PitchClass, Semitones};
pub use song::{
    AnnotationToken, ChordAlignment, ChordToken, Line, LineToken, LyricToken, Section,
    SectionLabel, Song, SongBuilder, SongId, SongSource, SourceFormat, Tempo, TimeSignature,
    Timestamp,
};
pub use transpose::{transpose_semitones, transpose_to_key};

/// Current product phase implemented by this crate's public surface.
pub const PHASE: u32 = 3;

/// Human-readable identity of the domain engine.
#[must_use]
pub fn engine_name() -> &'static str {
    "tonic-domain"
}

/// Semantic version of the domain crate, taken from Cargo.toml.
#[must_use]
pub fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Confirms the domain layer can execute without UI dependencies.
#[must_use]
pub fn is_available() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_is_available_without_ui() {
        assert!(is_available());
        assert_eq!(engine_name(), "tonic-domain");
        assert!(!engine_version().is_empty());
        assert_eq!(PHASE, 3);
    }
}
