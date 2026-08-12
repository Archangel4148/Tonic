//! Library list DTOs and search/filter matching.

use serde::{Deserialize, Serialize};
use tonic_domain::Song;
use tonic_persist::StoredSong;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInfoView {
    pub library_path: Option<String>,
    pub song_count: usize,
    pub setlist_count: usize,
    pub persistence_healthy: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryQuery {
    pub search: Option<String>,
    pub artist: Option<String>,
    pub key: Option<String>,
    pub favorites_only: Option<bool>,
    pub tag: Option<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySongSummary {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub original_key: Option<String>,
    pub performance_key: Option<String>,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub last_opened_at: Option<i64>,
    pub last_modified_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryListView {
    pub songs: Vec<LibrarySongSummary>,
    pub recents: Vec<LibrarySongSummary>,
    pub artists: Vec<String>,
    pub keys: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataUpdate {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

impl LibrarySongSummary {
    #[must_use]
    pub fn from_record(record: &StoredSong) -> Self {
        let song = &record.song;
        Self {
            id: song.id().as_str().to_string(),
            title: song.title().to_string(),
            artist: song.artist().map(str::to_string),
            album: song.album().map(str::to_string),
            original_key: song.original_key().map(|key| key.symbol()),
            performance_key: song.performance_key().map(|key| key.symbol()),
            favorite: record.favorite,
            tags: record.tags.clone(),
            last_opened_at: record.last_opened_at,
            last_modified_at: record.last_modified_at,
        }
    }
}

#[must_use]
pub fn build_list(records: &[StoredSong], query: &LibraryQuery) -> LibraryListView {
    let mut artists: Vec<String> = records
        .iter()
        .filter_map(|record| record.song.artist().map(str::to_string))
        .collect();
    uniq_sorted(&mut artists);

    let mut keys: Vec<String> = records
        .iter()
        .flat_map(|record| {
            [
                record.song.original_key().map(|key| key.symbol()),
                record.song.performance_key().map(|key| key.symbol()),
            ]
        })
        .flatten()
        .collect();
    uniq_sorted(&mut keys);

    let mut tags: Vec<String> = records
        .iter()
        .flat_map(|record| record.tags.iter().cloned())
        .collect();
    uniq_sorted(&mut tags);

    let mut recents: Vec<&StoredSong> = records
        .iter()
        .filter(|record| record.last_opened_at.is_some())
        .collect();
    recents.sort_by(|left, right| {
        right
            .last_opened_at
            .cmp(&left.last_opened_at)
            .then_with(|| title_cmp(&left.song, &right.song))
    });
    recents.truncate(8);

    let mut songs: Vec<&StoredSong> = records
        .iter()
        .filter(|record| matches_query(record, query))
        .collect();
    sort_records(&mut songs, query.sort.as_deref());

    LibraryListView {
        songs: songs
            .into_iter()
            .map(LibrarySongSummary::from_record)
            .collect(),
        recents: recents
            .into_iter()
            .map(LibrarySongSummary::from_record)
            .collect(),
        artists,
        keys,
        tags,
    }
}

#[must_use]
pub fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if out
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(trimmed))
        {
            continue;
        }
        out.push(trimmed.to_string());
    }
    out
}

#[must_use]
pub fn blank_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn matches_query(record: &StoredSong, query: &LibraryQuery) -> bool {
    if query.favorites_only.unwrap_or(false) && !record.favorite {
        return false;
    }
    if let Some(artist) = query
        .artist
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !record
            .song
            .artist()
            .is_some_and(|value| value.eq_ignore_ascii_case(artist))
        {
            return false;
        }
    }
    if let Some(key) = query
        .key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let original = record.song.original_key().map(|value| value.symbol());
        let performance = record.song.performance_key().map(|value| value.symbol());
        if original.as_deref() != Some(key) && performance.as_deref() != Some(key) {
            return false;
        }
    }
    if let Some(tag) = query
        .tag
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !record
            .tags
            .iter()
            .any(|value| value.eq_ignore_ascii_case(tag))
        {
            return false;
        }
    }
    if let Some(search) = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let needle = search.to_lowercase();
        let in_title = record.song.title().to_lowercase().contains(&needle);
        let in_artist = record
            .song
            .artist()
            .is_some_and(|artist| artist.to_lowercase().contains(&needle));
        let in_album = record
            .song
            .album()
            .is_some_and(|album| album.to_lowercase().contains(&needle));
        let in_tags = record
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&needle));
        let in_lyrics = song_lyrics(&record.song).to_lowercase().contains(&needle);
        if !(in_title || in_artist || in_album || in_tags || in_lyrics) {
            return false;
        }
    }
    true
}

fn sort_records(songs: &mut [&StoredSong], sort: Option<&str>) {
    match sort {
        Some("artist") => songs.sort_by(|left, right| {
            artist_key(&left.song)
                .cmp(&artist_key(&right.song))
                .then_with(|| title_cmp(&left.song, &right.song))
        }),
        Some("recentOpened") => songs.sort_by(|left, right| {
            right
                .last_opened_at
                .cmp(&left.last_opened_at)
                .then_with(|| title_cmp(&left.song, &right.song))
        }),
        Some("recentModified") => songs.sort_by(|left, right| {
            right
                .last_modified_at
                .cmp(&left.last_modified_at)
                .then_with(|| title_cmp(&left.song, &right.song))
        }),
        _ => songs.sort_by(|left, right| title_cmp(&left.song, &right.song)),
    }
}

fn title_cmp(left: &Song, right: &Song) -> std::cmp::Ordering {
    left.title()
        .to_lowercase()
        .cmp(&right.title().to_lowercase())
        .then_with(|| left.id().as_str().cmp(right.id().as_str()))
}

fn artist_key(song: &Song) -> String {
    song.artist()
        .map(str::to_lowercase)
        .unwrap_or_else(|| "\u{ffff}".to_string())
}

fn song_lyrics(song: &Song) -> String {
    song.sections()
        .iter()
        .flat_map(|section| section.lines().iter().map(|line| line.lyric_text()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn uniq_sorted(values: &mut Vec<String>) {
    values.sort_by_key(|left| left.to_lowercase());
    values.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
}
