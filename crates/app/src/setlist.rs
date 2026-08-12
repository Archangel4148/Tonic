//! Setlist DTOs. Entries reference library song ids.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tonic_domain::{played_key, Capo, Key};
use tonic_persist::{SetlistEntry, StoredSetlist, StoredSong};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistSummaryView {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
    pub event_date: Option<String>,
    pub song_count: u32,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistView {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
    pub event_date: Option<String>,
    pub entries: Vec<SetlistEntryView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistEntryView {
    pub id: String,
    pub song_id: String,
    pub title: String,
    pub artist: Option<String>,
    pub missing: bool,
    pub song_key: Option<String>,
    pub performance_key: Option<String>,
    pub capo_fret: Option<u8>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistContextView {
    pub setlist_id: String,
    pub setlist_name: String,
    pub entry_id: String,
    pub index: u32,
    pub total: u32,
    pub capo_fret: Option<u8>,
    pub entry_notes: Option<String>,
    pub played_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistMetaUpdate {
    pub name: String,
    pub notes: Option<String>,
    pub event_date: Option<String>,
}

#[must_use]
pub fn summary(setlist: &StoredSetlist) -> SetlistSummaryView {
    SetlistSummaryView {
        id: setlist.id.clone(),
        name: setlist.name.clone(),
        notes: setlist.notes.clone(),
        event_date: setlist.event_date.clone(),
        song_count: setlist.entries.len() as u32,
        updated_at: setlist.updated_at,
    }
}

#[must_use]
pub fn detail(setlist: &StoredSetlist, songs: &HashMap<String, StoredSong>) -> SetlistView {
    SetlistView {
        id: setlist.id.clone(),
        name: setlist.name.clone(),
        notes: setlist.notes.clone(),
        event_date: setlist.event_date.clone(),
        entries: setlist
            .entries
            .iter()
            .map(|entry| entry_view(entry, songs))
            .collect(),
    }
}

fn entry_view(entry: &SetlistEntry, songs: &HashMap<String, StoredSong>) -> SetlistEntryView {
    match songs.get(&entry.song_id) {
        Some(record) => SetlistEntryView {
            id: entry.id.clone(),
            song_id: entry.song_id.clone(),
            title: record.song.title().to_string(),
            artist: record.song.artist().map(str::to_string),
            missing: false,
            song_key: record
                .song
                .performance_key()
                .or_else(|| record.song.original_key())
                .map(|key| key.symbol()),
            performance_key: entry.performance_key.clone(),
            capo_fret: entry.capo_fret,
            notes: entry.notes.clone(),
        },
        None => SetlistEntryView {
            id: entry.id.clone(),
            song_id: entry.song_id.clone(),
            title: "(missing song)".to_string(),
            artist: None,
            missing: true,
            song_key: None,
            performance_key: entry.performance_key.clone(),
            capo_fret: entry.capo_fret,
            notes: entry.notes.clone(),
        },
    }
}

#[must_use]
pub fn context(
    setlist: &StoredSetlist,
    entry_id: &str,
    performance_key: Option<Key>,
) -> Option<SetlistContextView> {
    let index = setlist
        .entries
        .iter()
        .position(|entry| entry.id == entry_id)?;
    let entry = &setlist.entries[index];
    let played = match (performance_key, entry.capo_fret) {
        (Some(key), Some(fret)) => Capo::new(fret)
            .ok()
            .map(|capo| played_key(key, capo).symbol()),
        _ => None,
    };
    Some(SetlistContextView {
        setlist_id: setlist.id.clone(),
        setlist_name: setlist.name.clone(),
        entry_id: entry.id.clone(),
        index: index as u32,
        total: setlist.entries.len() as u32,
        capo_fret: entry.capo_fret,
        entry_notes: entry.notes.clone(),
        played_key: played,
    })
}
