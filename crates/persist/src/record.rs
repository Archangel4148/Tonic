//! On-disk library record. Domain [`Song`] plus library-only fields.

use serde::{Deserialize, Serialize};
use tonic_domain::Song;

/// How a performance key is realized on guitar.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransposeMode {
    /// Rewrite chord symbols into the sounding key.
    #[default]
    Chords,
    /// Keep written shapes and move a capo so the song sounds in the new key.
    Capo,
}

/// One library entry. Favorite, tags, and recents live here, not on [`Song`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSong {
    pub song: Song,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub last_opened_at: Option<i64>,
    #[serde(default)]
    pub last_modified_at: Option<i64>,
    #[serde(default)]
    pub transpose_mode: TransposeMode,
    #[serde(default)]
    pub capo_fret: Option<u8>,
}
