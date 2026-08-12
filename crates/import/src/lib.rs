//! ChordPro and plain-text import into Tonic's canonical [`Song`] model.
//!
//! This crate depends on `tonic-domain` only. It does not own music theory
//! or UI. Malformed input yields warnings and a usable [`Song`].

mod chordpro;
mod detect;
mod plain;
mod warning;

pub use detect::{detect_format, format_from_extension};
pub use warning::{ImportWarning, WarningKind, UNRECOGNIZED_CONTENT_MESSAGE};

use tonic_domain::{Song, SongId};

use chordpro::import_chordpro;
use plain::import_plain_text;

/// Supported Phase 4 import formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportFormat {
    ChordPro,
    PlainText,
}

/// Song plus non-fatal warnings. Import never discards usable content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportResult {
    pub song: Song,
    pub warnings: Vec<ImportWarning>,
}

impl ImportResult {
    #[must_use]
    pub fn has_issues(&self) -> bool {
        !self.warnings.is_empty()
    }

    #[must_use]
    pub fn summary_message(&self) -> Option<&'static str> {
        self.has_issues().then_some(UNRECOGNIZED_CONTENT_MESSAGE)
    }
}

/// Import `input` as the given format.
#[must_use]
pub fn import(input: &str, format: ImportFormat, id: impl Into<SongId>) -> ImportResult {
    match format {
        ImportFormat::ChordPro => import_chordpro(input, id),
        ImportFormat::PlainText => import_plain_text(input, id),
    }
}

/// Detect format from content, then import.
#[must_use]
pub fn import_auto(input: &str, id: impl Into<SongId>) -> ImportResult {
    import(input, detect_format(input), id)
}
