//! ChordPro, plain-text, MusicXML, and website URL import into Tonic's
//! canonical [`Song`] model.
//!
//! This crate depends on `tonic-domain` only (plus serde for web page JSON).
//! It does not own music theory or UI. Malformed input yields warnings and a
//! usable [`Song`].

mod chordpro;
mod detect;
mod musicxml;
mod plain;
mod section;
mod warning;
mod web;

pub use detect::{detect_format, format_from_extension};
pub use musicxml::{import_musicxml, import_musicxml_bytes};
pub use warning::{
    ImportWarning, WarningKind, UNRECOGNIZED_CONTENT_MESSAGE, UNSUPPORTED_MUSICXML_MESSAGE,
};
pub use web::{import_web_html, recognize_web_url, supported_web_sites, WebImportError, WebSite};

use tonic_domain::{Song, SongId};

use chordpro::import_chordpro;
use musicxml::{is_mxl_bytes, looks_like_musicxml};
pub use plain::export_plain_text;
use plain::import_plain_text;

/// Supported import formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportFormat {
    ChordPro,
    PlainText,
    MusicXml,
}

/// Song plus non-fatal warnings. Import never discards usable content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportResult {
    pub song: Song,
    pub warnings: Vec<ImportWarning>,
    /// Song-level capo from `{capo}` / UG `meta.capo`. Not a written chord.
    pub capo_fret: Option<u8>,
}

impl ImportResult {
    #[must_use]
    pub fn new(song: Song, warnings: Vec<ImportWarning>) -> Self {
        Self {
            song,
            warnings,
            capo_fret: None,
        }
    }

    #[must_use]
    pub fn has_issues(&self) -> bool {
        !self.warnings.is_empty()
    }

    #[must_use]
    pub fn summary_message(&self) -> Option<&'static str> {
        if self.warnings.is_empty() {
            None
        } else if self
            .warnings
            .iter()
            .all(|warning| warning.kind == WarningKind::UnsupportedFeature)
        {
            Some(UNSUPPORTED_MUSICXML_MESSAGE)
        } else {
            Some(UNRECOGNIZED_CONTENT_MESSAGE)
        }
    }
}

/// Import `input` as the given format.
#[must_use]
pub fn import(input: &str, format: ImportFormat, id: impl Into<SongId>) -> ImportResult {
    match format {
        ImportFormat::ChordPro => import_chordpro(input, id),
        ImportFormat::PlainText => import_plain_text(input, id),
        ImportFormat::MusicXml => import_musicxml(input, id),
    }
}

/// Detect format from content, then import.
#[must_use]
pub fn import_auto(input: &str, id: impl Into<SongId>) -> ImportResult {
    import(input, detect_format(input), id)
}

/// Import UTF-8 chart/MusicXML text or a compressed `.mxl` payload.
#[must_use]
pub fn import_bytes(bytes: &[u8], file_name: Option<&str>, id: impl Into<SongId>) -> ImportResult {
    let name = file_name.unwrap_or("").to_ascii_lowercase();
    if is_mxl_bytes(bytes) || name.ends_with(".mxl") {
        return import_musicxml_bytes(bytes, file_name, id);
    }
    let text = String::from_utf8_lossy(bytes);
    if name.ends_with(".musicxml") || name.ends_with(".xml") || looks_like_musicxml(&text) {
        return import_musicxml(&text, id);
    }
    import_auto(&text, id)
}
