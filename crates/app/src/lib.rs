//! Application services and authoritative in-memory state ownership.
//!
//! The UI must not own domain data. Persistence is not the source of truth
//! for the running session. This crate orchestrates domain, import, and
//! persistence without depending on Tauri or React.

mod library;
mod view;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use tonic_domain::{
    engine_name, engine_version, Key, ParseStatus, Quality, Song, SongId, Timestamp,
};
use tonic_import::{import, import_auto, ImportFormat, ImportWarning};
use tonic_persist::{FileLibrary, MemoryLibrary, SongLibrary, StoredSong};

pub use library::{LibraryListView, LibraryQuery, LibrarySongSummary, MetadataUpdate};
pub use tonic_persist::PersistError;
pub use view::{
    performance_key_choices, ChordView, LineView, SectionView, SongSessionView, SongView,
    WarningView,
};

/// How to interpret pasted or loaded chart text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportMode {
    Auto,
    ChordPro,
    PlainText,
}

impl ImportMode {
    /// Parse UI/IPC format names (`auto`, `chordPro`, `plainText`).
    ///
    /// # Errors
    ///
    /// Unknown format name.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim() {
            "" | "auto" | "Auto" => Ok(Self::Auto),
            "chordPro" | "chordpro" | "ChordPro" | "cho" => Ok(Self::ChordPro),
            "plainText" | "plaintext" | "PlainText" | "txt" => Ok(Self::PlainText),
            other => Err(format!("Unknown import format '{other}'.")),
        }
    }
}

/// Snapshot of process-level application identity and engine status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub phase: u32,
    pub domain_engine: &'static str,
    pub domain_version: &'static str,
}

struct AppState {
    next_id: u64,
    songs: HashMap<String, StoredSong>,
    session_id: Option<String>,
    warnings: Vec<ImportWarning>,
    steps: i32,
}

/// In-process application services.
///
/// Phase 6 owns the library in memory and write-through persists each change.
pub struct AppServices {
    library: Box<dyn SongLibrary>,
    state: Mutex<AppState>,
}

impl AppServices {
    #[must_use]
    pub fn new() -> Self {
        Self::in_memory()
    }

    #[must_use]
    pub fn in_memory() -> Self {
        Self::from_library(Box::new(MemoryLibrary::new())).expect("memory library loads")
    }

    /// Open a filesystem library under `root`.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError`] when the directory cannot be used.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, PersistError> {
        Self::from_library(Box::new(FileLibrary::open(root)?))
    }

    fn from_library(library: Box<dyn SongLibrary>) -> Result<Self, PersistError> {
        library.health_check()?;
        let (next_id, records) = library.load_all()?;
        let songs = records
            .into_iter()
            .map(|record| (record.song.id().as_str().to_string(), record))
            .collect();
        Ok(Self {
            library,
            state: Mutex::new(AppState {
                next_id: next_id.max(1),
                songs,
                session_id: None,
                warnings: Vec::new(),
                steps: 0,
            }),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, AppState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[must_use]
    pub fn info(&self) -> AppInfo {
        AppInfo {
            name: "Tonic",
            version: env!("CARGO_PKG_VERSION"),
            phase: 6,
            domain_engine: engine_name(),
            domain_version: engine_version(),
        }
    }

    #[must_use]
    pub fn persistence_healthy(&self) -> bool {
        self.library.health_check().is_ok()
    }

    #[must_use]
    pub fn list_library(&self, query: LibraryQuery) -> LibraryListView {
        let state = self.lock();
        let records: Vec<StoredSong> = state.songs.values().cloned().collect();
        library::build_list(&records, &query)
    }

    /// Import a chord sheet, save it to the library, and open it.
    ///
    /// # Errors
    ///
    /// Persistence write failure.
    pub fn import_text(&self, input: &str, mode: ImportMode) -> Result<SongSessionView, String> {
        let mut state = self.lock();
        let id = SongId::new(format!("song-{}", state.next_id));
        state.next_id += 1;
        let result = match mode {
            ImportMode::Auto => import_auto(input, id),
            ImportMode::ChordPro => import(input, ImportFormat::ChordPro, id),
            ImportMode::PlainText => import(input, ImportFormat::PlainText, id),
        };
        let now = Timestamp::now().as_secs();
        let mut song = result.song;
        if song.created_at().is_none() {
            song.set_created_at(Some(Timestamp::now()));
        }
        song.set_updated_at(Some(Timestamp::now()));
        let record = StoredSong {
            song,
            favorite: false,
            tags: Vec::new(),
            last_opened_at: Some(now),
            last_modified_at: Some(now),
        };
        self.persist_record(&record, Some(state.next_id))?;
        let song_id = record.song.id().as_str().to_string();
        state.songs.insert(song_id.clone(), record);
        state.warnings = result.warnings;
        state.steps = 0;
        state.session_id = Some(song_id);
        Ok(session_view(&state))
    }

    #[must_use]
    pub fn current_session(&self) -> Option<SongSessionView> {
        let state = self.lock();
        state.session_id.is_some().then(|| session_view(&state))
    }

    /// Open a library song into the session.
    ///
    /// # Errors
    ///
    /// Unknown id or persist failure updating recents.
    pub fn open_song(&self, id: &str) -> Result<SongSessionView, String> {
        let mut state = self.lock();
        let record = state
            .songs
            .get_mut(id)
            .ok_or_else(|| format!("Song '{id}' was not found."))?;
        record.last_opened_at = Some(Timestamp::now().as_secs());
        let persisted = record.clone();
        self.persist_record(&persisted, None)?;
        state.warnings.clear();
        state.steps = steps_from_song(&persisted.song);
        state.session_id = Some(id.to_string());
        Ok(session_view(&state))
    }

    /// Shift performance key by `semitones`. Infers original key if missing.
    ///
    /// # Errors
    ///
    /// No song loaded, or persist failure.
    pub fn transpose_by(&self, semitones: i32) -> Result<SongSessionView, String> {
        let mut state = self.lock();
        {
            let song = open_song_mut(&mut state)?;
            ensure_original_key(song);
        }
        state.steps += semitones;
        apply_steps(&mut state);
        self.persist_open(&mut state)?;
        Ok(session_view(&state))
    }

    /// Set the performance key by symbol (`D`, `F#m`, …).
    ///
    /// # Errors
    ///
    /// No song loaded, invalid key, or persist failure.
    pub fn set_performance_key(&self, symbol: &str) -> Result<SongSessionView, String> {
        let target = Key::parse(symbol).ok_or_else(|| format!("Unknown key '{symbol}'."))?;
        let mut state = self.lock();
        let original = {
            let song = open_song_mut(&mut state)?;
            ensure_original_key(song)
        };
        let diff =
            i32::from(target.pitch_class().value()) - i32::from(original.pitch_class().value());
        state.steps = diff.rem_euclid(12);
        open_song_mut(&mut state)?.set_performance_key(Some(target));
        self.persist_open(&mut state)?;
        Ok(session_view(&state))
    }

    /// Restore performance key to the original key.
    ///
    /// # Errors
    ///
    /// No song loaded, or persist failure.
    pub fn reset_performance_key(&self) -> Result<SongSessionView, String> {
        let mut state = self.lock();
        {
            let song = open_song_mut(&mut state)?;
            if let Some(original) = song.original_key() {
                song.set_performance_key(Some(original));
            }
        }
        state.steps = 0;
        self.persist_open(&mut state)?;
        Ok(session_view(&state))
    }

    pub fn close_song(&self) {
        let mut state = self.lock();
        state.session_id = None;
        state.warnings.clear();
        state.steps = 0;
    }

    /// Toggle favorite on a library song.
    ///
    /// # Errors
    ///
    /// Unknown id or persist failure.
    pub fn toggle_favorite(&self, id: &str) -> Result<Option<SongSessionView>, String> {
        let mut state = self.lock();
        let record = state
            .songs
            .get_mut(id)
            .ok_or_else(|| format!("Song '{id}' was not found."))?;
        record.favorite = !record.favorite;
        let persisted = record.clone();
        self.persist_record(&persisted, None)?;
        Ok(state
            .session_id
            .as_deref()
            .filter(|open| *open == id)
            .map(|_| session_view(&state)))
    }

    /// Update metadata on the open song.
    ///
    /// # Errors
    ///
    /// No song loaded, empty title, or persist failure.
    pub fn update_open_metadata(&self, update: MetadataUpdate) -> Result<SongSessionView, String> {
        let title = update.title.trim();
        if title.is_empty() {
            return Err("Title cannot be empty.".to_string());
        }
        let tags = library::normalize_tags(update.tags);
        let mut state = self.lock();
        {
            let song = open_song_mut(&mut state)?;
            song.set_title(title);
            song.set_artist(library::blank_to_none(update.artist));
            song.set_album(library::blank_to_none(update.album));
            song.set_notes(library::blank_to_none(update.notes));
        }
        let id = state
            .session_id
            .clone()
            .ok_or_else(|| "No song is loaded.".to_string())?;
        if let Some(record) = state.songs.get_mut(&id) {
            record.tags = tags;
        }
        self.persist_open(&mut state)?;
        Ok(session_view(&state))
    }

    /// Duplicate a library song and open the copy.
    ///
    /// # Errors
    ///
    /// Unknown id or persist failure.
    pub fn duplicate_song(&self, id: &str) -> Result<SongSessionView, String> {
        let mut state = self.lock();
        let source = state
            .songs
            .get(id)
            .ok_or_else(|| format!("Song '{id}' was not found."))?
            .clone();
        let new_id = format!("song-{}", state.next_id);
        state.next_id += 1;
        let now = Timestamp::now();
        let mut song = source.song;
        song.set_id(SongId::new(new_id.clone()));
        song.set_title(format!("{} (copy)", song.title()));
        song.set_created_at(Some(now));
        song.set_updated_at(Some(now));
        let record = StoredSong {
            song,
            favorite: false,
            tags: source.tags,
            last_opened_at: Some(now.as_secs()),
            last_modified_at: Some(now.as_secs()),
        };
        self.persist_record(&record, Some(state.next_id))?;
        state.songs.insert(new_id.clone(), record);
        state.warnings.clear();
        state.steps = 0;
        if let Some(copy) = state.songs.get(&new_id) {
            state.steps = steps_from_song(&copy.song);
        }
        state.session_id = Some(new_id);
        Ok(session_view(&state))
    }

    /// Delete a library song.
    ///
    /// # Errors
    ///
    /// Unknown id or persist failure.
    pub fn delete_song(&self, id: &str) -> Result<Option<SongSessionView>, String> {
        let mut state = self.lock();
        if !state.songs.contains_key(id) {
            return Err(format!("Song '{id}' was not found."));
        }
        self.library.delete(id).map_err(|error| error.to_string())?;
        state.songs.remove(id);
        if state.session_id.as_deref() == Some(id) {
            state.session_id = None;
            state.warnings.clear();
            state.steps = 0;
        }
        Ok(state.session_id.as_ref().map(|_| session_view(&state)))
    }

    fn persist_open(&self, state: &mut AppState) -> Result<(), String> {
        let id = state
            .session_id
            .as_deref()
            .ok_or_else(|| "No song is loaded.".to_string())?
            .to_string();
        let now = Timestamp::now();
        let record = state
            .songs
            .get_mut(&id)
            .ok_or_else(|| "No song is loaded.".to_string())?;
        record.song.set_updated_at(Some(now));
        record.last_modified_at = Some(now.as_secs());
        let persisted = record.clone();
        self.persist_record(&persisted, None)
    }

    fn persist_record(&self, record: &StoredSong, next_id: Option<u64>) -> Result<(), String> {
        self.library
            .save(record)
            .map_err(|error| error.to_string())?;
        if let Some(next_id) = next_id {
            self.library
                .save_next_id(next_id)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

impl Default for AppServices {
    fn default() -> Self {
        Self::new()
    }
}

fn session_view(state: &AppState) -> SongSessionView {
    let id = state
        .session_id
        .as_ref()
        .expect("session view requires a song");
    let record = state
        .songs
        .get(id)
        .expect("open song must exist in library");
    SongSessionView::from_parts(
        &record.song,
        &state.warnings,
        state.steps,
        record.favorite,
        record.tags.clone(),
    )
}

fn open_song_mut(state: &mut AppState) -> Result<&mut Song, String> {
    let id = state
        .session_id
        .as_ref()
        .ok_or_else(|| "No song is loaded.".to_string())?
        .clone();
    state
        .songs
        .get_mut(&id)
        .map(|record| &mut record.song)
        .ok_or_else(|| "No song is loaded.".to_string())
}

fn apply_steps(state: &mut AppState) {
    let steps = state.steps;
    let Ok(song) = open_song_mut(state) else {
        return;
    };
    let Some(original) = song.original_key() else {
        return;
    };
    song.set_performance_key(Some(original.transpose_semitones(steps)));
}

fn ensure_original_key(song: &mut Song) -> Key {
    if let Some(key) = song.original_key() {
        if song.performance_key().is_none() {
            song.set_performance_key(Some(key));
        }
        return key;
    }
    let inferred = infer_key(song);
    song.set_original_key(Some(inferred));
    song.set_performance_key(Some(inferred));
    inferred
}

fn infer_key(song: &Song) -> Key {
    for section in song.sections() {
        for line in section.lines() {
            for token in line.chord_tokens() {
                let chord = token.chord();
                if chord.status() != ParseStatus::FullyRecognized {
                    continue;
                }
                if let Some(root) = chord.root() {
                    return match chord.quality() {
                        Quality::Minor => Key::minor(root),
                        _ => Key::major(root),
                    };
                }
            }
        }
    }
    Key::parse("C").expect("C is a valid key")
}

fn steps_from_song(song: &Song) -> i32 {
    match (song.original_key(), song.performance_key()) {
        (Some(original), Some(performance)) => (i32::from(performance.pitch_class().value())
            - i32::from(original.pitch_class().value()))
        .rem_euclid(12),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic_import::UNRECOGNIZED_CONTENT_MESSAGE;

    fn temp_root() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tonic-app-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn services_report_healthy_stack() {
        let services = AppServices::new();
        let info = services.info();

        assert_eq!(info.name, "Tonic");
        assert_eq!(info.phase, 6);
        assert_eq!(info.domain_engine, "tonic-domain");
        assert!(!info.version.is_empty());
        assert!(!info.domain_version.is_empty());
        assert!(services.persistence_healthy());
    }

    #[test]
    fn import_stores_session_and_builds_view() {
        let services = AppServices::new();
        let session = services
            .import_text(
                "{title: Demo}\n{key: C}\n[C]Hi [G]there",
                ImportMode::ChordPro,
            )
            .unwrap();
        assert_eq!(session.song.title, "Demo");
        assert_eq!(session.song.id, "song-1");
        assert_eq!(session.song.original_key.as_deref(), Some("C"));
        assert_eq!(session.song.performance_key.as_deref(), Some("C"));
        assert_eq!(session.song.sections[0].lines[0].lyrics, "Hi there");
        assert_eq!(session.song.sections[0].lines[0].chords[0].symbol, "C");
        assert!(!session.favorite);
        assert!(session.tags.is_empty());
        assert_eq!(services.current_session().unwrap().song.id, session.song.id);
        let list = services.list_library(LibraryQuery::default());
        assert_eq!(list.songs.len(), 1);
        assert_eq!(list.songs[0].title, "Demo");
    }

    #[test]
    fn transpose_updates_display_not_written_chords_or_source() {
        let services = AppServices::new();
        let _imported = services
            .import_text(
                "{title: Demo}\n{key: C}\n[C]Hi [G]there",
                ImportMode::ChordPro,
            )
            .unwrap();
        let source_before = services
            .current_session()
            .unwrap()
            .song
            .source_format
            .clone();

        let up = services.transpose_by(2).unwrap();
        assert_eq!(up.song.performance_key.as_deref(), Some("D"));
        assert_eq!(up.song.original_key.as_deref(), Some("C"));
        assert_eq!(up.semitone_offset, 2);
        let line = &up.song.sections[0].lines[0];
        assert_eq!(line.chords[0].symbol, "D");
        assert_eq!(line.chords[0].written, "C");
        assert_eq!(line.chords[1].symbol, "A");
        assert_eq!(line.chords[1].written, "G");
        assert_eq!(line.lyrics, "Hi there");
        assert_eq!(up.song.source_format, source_before);

        let reset = services.reset_performance_key().unwrap();
        assert_eq!(reset.semitone_offset, 0);
        assert_eq!(reset.song.performance_key.as_deref(), Some("C"));
        assert_eq!(reset.song.sections[0].lines[0].chords[0].symbol, "C");
    }

    #[test]
    fn transpose_infers_original_key_from_first_chord() {
        let services = AppServices::new();
        let imported = services
            .import_text("[Am]Hello [E]world", ImportMode::ChordPro)
            .unwrap();
        assert!(imported.song.original_key.is_none());

        let up = services.transpose_by(2).unwrap();
        assert_eq!(up.song.original_key.as_deref(), Some("Am"));
        assert_eq!(up.song.performance_key.as_deref(), Some("Bm"));
        assert_eq!(up.song.sections[0].lines[0].chords[0].symbol, "Bm");
        assert_eq!(up.song.sections[0].lines[0].chords[0].written, "Am");
    }

    #[test]
    fn set_performance_key_and_close() {
        let services = AppServices::new();
        let _imported = services
            .import_text("{title: X}\n{key: G}\n[G]Hi", ImportMode::ChordPro)
            .unwrap();
        let session = services.set_performance_key("A").unwrap();
        assert_eq!(session.song.performance_key.as_deref(), Some("A"));
        assert_eq!(session.song.sections[0].lines[0].chords[0].symbol, "A");
        services.close_song();
        assert!(services.current_session().is_none());
        assert_eq!(services.transpose_by(1).unwrap_err(), "No song is loaded.");
        assert_eq!(
            services.list_library(LibraryQuery::default()).songs.len(),
            1
        );
    }

    #[test]
    fn import_warnings_surface_summary() {
        let services = AppServices::new();
        let session = services
            .import_text("{title: X}\n[C]Hi [Xyz]there", ImportMode::ChordPro)
            .unwrap();
        assert!(session.summary_message.as_deref() == Some(UNRECOGNIZED_CONTENT_MESSAGE));
        assert!(!session.warnings.is_empty());
        assert_eq!(
            session.song.sections[0].lines[0].chords[1].status,
            "unrecognized"
        );
        assert_eq!(session.song.sections[0].lines[0].chords[1].symbol, "Xyz");
    }

    #[test]
    fn library_round_trips_across_reopen() {
        let root = temp_root();
        {
            let services = AppServices::open(&root).unwrap();
            let _imported = services
                .import_text(
                    "{title: Grace}\n{artist: Traditional}\n{key: G}\n[G]Amazing [D]grace",
                    ImportMode::ChordPro,
                )
                .unwrap();
            services.transpose_by(2).unwrap();
            services.toggle_favorite("song-1").unwrap();
            services
                .update_open_metadata(MetadataUpdate {
                    title: "Amazing Grace".into(),
                    artist: Some("Traditional".into()),
                    album: Some("Hymns".into()),
                    notes: Some("Sunday".into()),
                    tags: vec!["gospel".into(), " gospel ".into(), "".into()],
                })
                .unwrap();
        }

        let services = AppServices::open(&root).unwrap();
        let list = services.list_library(LibraryQuery::default());
        assert_eq!(list.songs.len(), 1);
        assert_eq!(list.songs[0].title, "Amazing Grace");
        assert_eq!(list.songs[0].artist.as_deref(), Some("Traditional"));
        assert_eq!(list.songs[0].album.as_deref(), Some("Hymns"));
        assert_eq!(list.songs[0].performance_key.as_deref(), Some("A"));
        assert!(list.songs[0].favorite);
        assert_eq!(list.songs[0].tags, ["gospel"]);
        assert_eq!(list.tags, ["gospel"]);
        assert!(list.artists.iter().any(|artist| artist == "Traditional"));

        let opened = services.open_song("song-1").unwrap();
        assert_eq!(opened.song.title, "Amazing Grace");
        assert_eq!(opened.song.performance_key.as_deref(), Some("A"));
        assert_eq!(opened.semitone_offset, 2);
        assert_eq!(opened.tags, ["gospel"]);
        assert!(opened.favorite);

        let searched = services.list_library(LibraryQuery {
            search: Some("amazing".into()),
            ..LibraryQuery::default()
        });
        assert_eq!(searched.songs.len(), 1);
        let lyric_hit = services.list_library(LibraryQuery {
            search: Some("grace".into()),
            ..LibraryQuery::default()
        });
        assert_eq!(lyric_hit.songs.len(), 1);
        let miss = services.list_library(LibraryQuery {
            search: Some("zzz".into()),
            ..LibraryQuery::default()
        });
        assert!(miss.songs.is_empty());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn duplicate_and_delete_library_songs() {
        let services = AppServices::new();
        let _imported = services
            .import_text("{title: Original}\n{key: C}\n[C]Hi", ImportMode::ChordPro)
            .unwrap();
        services.toggle_favorite("song-1").unwrap();
        let copy = services.duplicate_song("song-1").unwrap();
        assert_eq!(copy.song.title, "Original (copy)");
        assert_eq!(copy.song.id, "song-2");
        assert!(!copy.favorite);
        assert_eq!(
            services.list_library(LibraryQuery::default()).songs.len(),
            2
        );

        let remaining = services.delete_song("song-2").unwrap();
        assert!(remaining.is_none());
        assert!(services.current_session().is_none());
        assert_eq!(
            services.list_library(LibraryQuery::default()).songs.len(),
            1
        );
        assert_eq!(
            services.list_library(LibraryQuery::default()).songs[0].id,
            "song-1"
        );
    }
}
