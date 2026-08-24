//! Durable setlist records. Entries reference song ids; they never copy songs.

use serde::{Deserialize, Serialize};

use crate::record::TransposeMode;

/// One performance slot in a setlist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistEntry {
    pub id: String,
    pub song_id: String,
    #[serde(default)]
    pub performance_key: Option<String>,
    #[serde(default)]
    pub capo_fret: Option<u8>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub transpose_mode: TransposeMode,
    /// Fingered key for capo mode. `None` means use the song's original key.
    #[serde(default)]
    pub shapes_key: Option<String>,
}

/// Named ordered list of song references plus optional event notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSetlist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub event_date: Option<String>,
    #[serde(default)]
    pub entries: Vec<SetlistEntry>,
    #[serde(default)]
    pub updated_at: Option<i64>,
}

/// Counters plus setlists loaded from disk.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SetlistSnapshot {
    pub next_setlist_id: u64,
    pub next_entry_id: u64,
    pub setlists: Vec<StoredSetlist>,
}
