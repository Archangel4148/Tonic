//! Application services and authoritative in-memory state ownership.
//!
//! The UI must not own domain data. Persistence is not the source of truth
//! for the running session. This crate orchestrates domain, import, and
//! persistence without depending on Tauri or React.

mod editor;
mod library;
mod setlist;
mod view;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use tonic_domain::{
    engine_name, engine_version, Key, Line, ParseStatus, Quality, Section, Song, SongId, Timestamp,
};
use tonic_import::{import, import_auto, ImportFormat, ImportWarning};
use tonic_persist::{
    FileLibrary, MemoryLibrary, SetlistEntry, SongLibrary, StoredSetlist, StoredSong,
};

use editor::{
    apply_meta, editor_view, line_mut, parse_chord_symbol, parse_label, refresh_chord_warnings,
    EditorSession,
};

pub use editor::{
    EditorChordView, EditorLineView, EditorMetaUpdate, EditorSaveResult, EditorSectionView,
    EditorSessionView, SectionLabelInput,
};
pub use library::{LibraryListView, LibraryQuery, LibrarySongSummary, MetadataUpdate};
pub use setlist::{
    SetlistContextView, SetlistEntryView, SetlistMetaUpdate, SetlistSummaryView, SetlistView,
};
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
    next_setlist_id: u64,
    next_entry_id: u64,
    songs: HashMap<String, StoredSong>,
    setlists: HashMap<String, StoredSetlist>,
    session_id: Option<String>,
    session_setlist_id: Option<String>,
    session_entry_id: Option<String>,
    warnings: Vec<ImportWarning>,
    steps: i32,
    editor: Option<EditorSession>,
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
        let snapshot = library.load_setlists()?;
        let songs = records
            .into_iter()
            .map(|record| (record.song.id().as_str().to_string(), record))
            .collect();
        let setlists = snapshot
            .setlists
            .into_iter()
            .map(|setlist| (setlist.id.clone(), setlist))
            .collect();
        Ok(Self {
            library,
            state: Mutex::new(AppState {
                next_id: next_id.max(1),
                next_setlist_id: snapshot.next_setlist_id.max(1),
                next_entry_id: snapshot.next_entry_id.max(1),
                songs,
                setlists,
                session_id: None,
                session_setlist_id: None,
                session_entry_id: None,
                warnings: Vec::new(),
                steps: 0,
                editor: None,
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
            phase: 9,
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
        ensure_no_dirty_editor(&state)?;
        state.editor = None;
        clear_setlist_session(&mut state);
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
        ensure_no_dirty_editor(&state)?;
        state.editor = None;
        clear_setlist_session(&mut state);
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
        ensure_no_dirty_editor(&state)?;
        let setlist_session = state.session_setlist_id.is_some();
        {
            let song = open_song_mut(&mut state)?;
            if setlist_session {
                ensure_original_key_only(song);
            } else {
                ensure_original_key(song);
            }
        }
        if setlist_session {
            self.persist_song_document(&state)?;
            state.steps += semitones;
            apply_setlist_steps(&mut state)?;
            self.persist_open_setlist(&mut state)?;
        } else {
            state.steps += semitones;
            apply_steps(&mut state);
            self.persist_open(&mut state)?;
        }
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
        ensure_no_dirty_editor(&state)?;
        let setlist_session = state.session_setlist_id.is_some();
        let original = {
            let song = open_song_mut(&mut state)?;
            if setlist_session {
                ensure_original_key_only(song)
            } else {
                ensure_original_key(song)
            }
        };
        let diff =
            i32::from(target.pitch_class().value()) - i32::from(original.pitch_class().value());
        state.steps = diff.rem_euclid(12);
        if setlist_session {
            self.persist_song_document(&state)?;
            set_open_entry_key(&mut state, Some(target.symbol()))?;
            self.persist_open_setlist(&mut state)?;
        } else {
            open_song_mut(&mut state)?.set_performance_key(Some(target));
            self.persist_open(&mut state)?;
        }
        Ok(session_view(&state))
    }

    /// Restore performance key to the original key.
    ///
    /// # Errors
    ///
    /// No song loaded, or persist failure.
    pub fn reset_performance_key(&self) -> Result<SongSessionView, String> {
        let mut state = self.lock();
        ensure_no_dirty_editor(&state)?;
        if state.session_setlist_id.is_some() {
            set_open_entry_key(&mut state, None)?;
            if let Some(song_id) = state.session_id.as_deref() {
                if let Some(record) = state.songs.get(song_id) {
                    state.steps = steps_from_song(&record.song);
                }
            }
            self.persist_open_setlist(&mut state)?;
        } else {
            {
                let song = open_song_mut(&mut state)?;
                if let Some(original) = song.original_key() {
                    song.set_performance_key(Some(original));
                }
            }
            state.steps = 0;
            self.persist_open(&mut state)?;
        }
        Ok(session_view(&state))
    }

    pub fn close_song(&self) -> Result<(), String> {
        let mut state = self.lock();
        ensure_no_dirty_editor(&state)?;
        state.editor = None;
        clear_setlist_session(&mut state);
        state.session_id = None;
        state.warnings.clear();
        state.steps = 0;
        Ok(())
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
        ensure_no_dirty_editor(&state)?;
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
        ensure_no_dirty_editor(&state)?;
        state.editor = None;
        clear_setlist_session(&mut state);
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
        if state
            .editor
            .as_ref()
            .is_some_and(|editor| editor.dirty && editor.draft.id().as_str() == id)
        {
            return Err("Save or cancel the editor first.".to_string());
        }
        self.library.delete(id).map_err(|error| error.to_string())?;
        state.songs.remove(id);
        if state
            .editor
            .as_ref()
            .is_some_and(|editor| editor.draft.id().as_str() == id)
        {
            state.editor = None;
        }
        if state.session_id.as_deref() == Some(id) {
            clear_setlist_session(&mut state);
            state.session_id = None;
            state.warnings.clear();
            state.steps = 0;
        }
        Ok(state.session_id.as_ref().map(|_| session_view(&state)))
    }

    #[must_use]
    pub fn list_setlists(&self) -> Vec<SetlistSummaryView> {
        let state = self.lock();
        let mut list: Vec<SetlistSummaryView> =
            state.setlists.values().map(setlist::summary).collect();
        list.sort_by_key(|left| left.name.to_lowercase());
        list
    }

    /// # Errors
    ///
    /// Unknown setlist id.
    pub fn get_setlist(&self, id: &str) -> Result<SetlistView, String> {
        let state = self.lock();
        let setlist = state
            .setlists
            .get(id)
            .ok_or_else(|| format!("Setlist '{id}' was not found."))?;
        Ok(setlist::detail(setlist, &state.songs))
    }

    /// # Errors
    ///
    /// Persist failure.
    pub fn create_setlist(&self, name: Option<String>) -> Result<SetlistView, String> {
        let mut state = self.lock();
        let id = format!("setlist-{}", state.next_setlist_id);
        state.next_setlist_id += 1;
        let name = name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "Untitled setlist".to_string());
        let setlist = StoredSetlist {
            id: id.clone(),
            name,
            notes: None,
            event_date: None,
            entries: Vec::new(),
            updated_at: Some(Timestamp::now().as_secs()),
        };
        self.persist_setlist(&setlist, Some((state.next_setlist_id, state.next_entry_id)))?;
        state.setlists.insert(id, setlist.clone());
        Ok(setlist::detail(&setlist, &state.songs))
    }

    /// # Errors
    ///
    /// Unknown id, empty name, or persist failure.
    pub fn update_setlist_meta(
        &self,
        id: &str,
        update: SetlistMetaUpdate,
    ) -> Result<SetlistView, String> {
        let name = update.name.trim();
        if name.is_empty() {
            return Err("Setlist name cannot be empty.".to_string());
        }
        let mut state = self.lock();
        let setlist = state
            .setlists
            .get_mut(id)
            .ok_or_else(|| format!("Setlist '{id}' was not found."))?;
        setlist.name = name.to_string();
        setlist.notes = library::blank_to_none(update.notes);
        setlist.event_date = library::blank_to_none(update.event_date);
        setlist.updated_at = Some(Timestamp::now().as_secs());
        let persisted = setlist.clone();
        self.persist_setlist(&persisted, None)?;
        Ok(setlist::detail(&persisted, &state.songs))
    }

    /// # Errors
    ///
    /// Unknown id or persist failure.
    pub fn delete_setlist(&self, id: &str) -> Result<(), String> {
        let mut state = self.lock();
        if !state.setlists.contains_key(id) {
            return Err(format!("Setlist '{id}' was not found."));
        }
        self.library
            .delete_setlist(id)
            .map_err(|error| error.to_string())?;
        state.setlists.remove(id);
        if state.session_setlist_id.as_deref() == Some(id) {
            clear_setlist_session(&mut state);
        }
        Ok(())
    }

    /// # Errors
    ///
    /// Unknown id or persist failure.
    pub fn duplicate_setlist(&self, id: &str) -> Result<SetlistView, String> {
        let mut state = self.lock();
        let source = state
            .setlists
            .get(id)
            .ok_or_else(|| format!("Setlist '{id}' was not found."))?
            .clone();
        let new_id = format!("setlist-{}", state.next_setlist_id);
        state.next_setlist_id += 1;
        let mut entries = Vec::new();
        for entry in source.entries {
            let entry_id = format!("entry-{}", state.next_entry_id);
            state.next_entry_id += 1;
            entries.push(SetlistEntry {
                id: entry_id,
                song_id: entry.song_id,
                performance_key: entry.performance_key,
                capo_fret: entry.capo_fret,
                notes: entry.notes,
            });
        }
        let setlist = StoredSetlist {
            id: new_id.clone(),
            name: format!("{} (copy)", source.name),
            notes: source.notes,
            event_date: source.event_date,
            entries,
            updated_at: Some(Timestamp::now().as_secs()),
        };
        self.persist_setlist(&setlist, Some((state.next_setlist_id, state.next_entry_id)))?;
        state.setlists.insert(new_id, setlist.clone());
        Ok(setlist::detail(&setlist, &state.songs))
    }

    /// # Errors
    ///
    /// Unknown setlist/song or persist failure.
    pub fn add_setlist_song(&self, setlist_id: &str, song_id: &str) -> Result<SetlistView, String> {
        let mut state = self.lock();
        if !state.songs.contains_key(song_id) {
            return Err(format!("Song '{song_id}' was not found."));
        }
        let entry_id = format!("entry-{}", state.next_entry_id);
        state.next_entry_id += 1;
        let setlist = state
            .setlists
            .get_mut(setlist_id)
            .ok_or_else(|| format!("Setlist '{setlist_id}' was not found."))?;
        setlist.entries.push(SetlistEntry {
            id: entry_id,
            song_id: song_id.to_string(),
            performance_key: None,
            capo_fret: None,
            notes: None,
        });
        setlist.updated_at = Some(Timestamp::now().as_secs());
        let persisted = setlist.clone();
        self.persist_setlist(
            &persisted,
            Some((state.next_setlist_id, state.next_entry_id)),
        )?;
        Ok(setlist::detail(&persisted, &state.songs))
    }

    /// # Errors
    ///
    /// Unknown setlist/entry or persist failure.
    pub fn remove_setlist_entry(
        &self,
        setlist_id: &str,
        entry_id: &str,
    ) -> Result<SetlistView, String> {
        let mut state = self.lock();
        let setlist = state
            .setlists
            .get_mut(setlist_id)
            .ok_or_else(|| format!("Setlist '{setlist_id}' was not found."))?;
        let before = setlist.entries.len();
        setlist.entries.retain(|entry| entry.id != entry_id);
        if setlist.entries.len() == before {
            return Err(format!("Setlist entry '{entry_id}' was not found."));
        }
        setlist.updated_at = Some(Timestamp::now().as_secs());
        let persisted = setlist.clone();
        self.persist_setlist(&persisted, None)?;
        if state.session_entry_id.as_deref() == Some(entry_id) {
            clear_setlist_session(&mut state);
        }
        Ok(setlist::detail(&persisted, &state.songs))
    }

    /// # Errors
    ///
    /// Unknown setlist/index or persist failure.
    pub fn move_setlist_entry(
        &self,
        setlist_id: &str,
        from: usize,
        to: usize,
    ) -> Result<SetlistView, String> {
        let mut state = self.lock();
        let setlist = state
            .setlists
            .get_mut(setlist_id)
            .ok_or_else(|| format!("Setlist '{setlist_id}' was not found."))?;
        let len = setlist.entries.len();
        if from >= len || to >= len {
            return Err("That setlist entry was not found.".to_string());
        }
        if from != to {
            let entry = setlist.entries.remove(from);
            setlist.entries.insert(to, entry);
            setlist.updated_at = Some(Timestamp::now().as_secs());
        }
        let persisted = setlist.clone();
        self.persist_setlist(&persisted, None)?;
        Ok(setlist::detail(&persisted, &state.songs))
    }

    /// # Errors
    ///
    /// Unknown entry, invalid capo, or persist failure.
    pub fn update_setlist_entry(
        &self,
        setlist_id: &str,
        entry_id: &str,
        performance_key: Option<String>,
        capo_fret: Option<u8>,
        notes: Option<String>,
    ) -> Result<SetlistView, String> {
        if let Some(fret) = capo_fret {
            tonic_domain::Capo::new(fret).map_err(|error| error.to_string())?;
        }
        let key = match performance_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(symbol) => Some(
                Key::parse(symbol)
                    .ok_or_else(|| format!("Unknown key '{symbol}'."))?
                    .symbol(),
            ),
            None => None,
        };
        let mut state = self.lock();
        let setlist = state
            .setlists
            .get_mut(setlist_id)
            .ok_or_else(|| format!("Setlist '{setlist_id}' was not found."))?;
        let entry = setlist
            .entries
            .iter_mut()
            .find(|entry| entry.id == entry_id)
            .ok_or_else(|| format!("Setlist entry '{entry_id}' was not found."))?;
        entry.performance_key = key;
        entry.capo_fret = capo_fret;
        entry.notes = library::blank_to_none(notes);
        setlist.updated_at = Some(Timestamp::now().as_secs());
        let persisted = setlist.clone();
        self.persist_setlist(&persisted, None)?;
        if state.session_entry_id.as_deref() == Some(entry_id) {
            if let Some(song_id) = state.session_id.clone() {
                if let Some(record) = state.songs.get(&song_id) {
                    state.steps = entry_steps(&record.song, &persisted, entry_id);
                }
            }
        }
        Ok(setlist::detail(&persisted, &state.songs))
    }

    /// Open a setlist entry without mutating the underlying song document.
    ///
    /// # Errors
    ///
    /// Unknown setlist/entry/song, dirty editor, or persist failure updating recents.
    pub fn open_setlist_entry(
        &self,
        setlist_id: &str,
        entry_id: &str,
    ) -> Result<SongSessionView, String> {
        let mut state = self.lock();
        ensure_no_dirty_editor(&state)?;
        state.editor = None;
        let setlist = state
            .setlists
            .get(setlist_id)
            .ok_or_else(|| format!("Setlist '{setlist_id}' was not found."))?
            .clone();
        let entry = setlist
            .entries
            .iter()
            .find(|entry| entry.id == entry_id)
            .ok_or_else(|| format!("Setlist entry '{entry_id}' was not found."))?
            .clone();
        let record = state
            .songs
            .get_mut(&entry.song_id)
            .ok_or_else(|| format!("Song '{}' was not found.", entry.song_id))?;
        record.last_opened_at = Some(Timestamp::now().as_secs());
        let persisted = record.clone();
        self.persist_record(&persisted, None)?;
        state.warnings.clear();
        state.session_id = Some(entry.song_id.clone());
        state.session_setlist_id = Some(setlist_id.to_string());
        state.session_entry_id = Some(entry_id.to_string());
        state.steps = entry_steps(&persisted.song, &setlist, entry_id);
        Ok(session_view(&state))
    }

    /// Open the next or previous playable setlist entry (skips missing songs).
    ///
    /// # Errors
    ///
    /// No setlist session, no remaining song in that direction, dirty editor, or persist failure.
    pub fn open_setlist_neighbor(&self, delta: i32) -> Result<SongSessionView, String> {
        let step = if delta < 0 { -1_isize } else { 1_isize };
        if delta == 0 {
            return Err("Setlist navigation needs a direction.".to_string());
        }
        let (setlist_id, entry_id) = {
            let state = self.lock();
            ensure_no_dirty_editor(&state)?;
            let setlist_id = state
                .session_setlist_id
                .clone()
                .ok_or_else(|| "No setlist is open.".to_string())?;
            let current_entry = state
                .session_entry_id
                .clone()
                .ok_or_else(|| "No setlist entry is open.".to_string())?;
            let setlist = state
                .setlists
                .get(&setlist_id)
                .ok_or_else(|| format!("Setlist '{setlist_id}' was not found."))?;
            let current = setlist
                .entries
                .iter()
                .position(|entry| entry.id == current_entry)
                .ok_or_else(|| "That setlist entry was not found.".to_string())?;
            let mut index = current as isize + step;
            let mut found = None;
            while index >= 0 && (index as usize) < setlist.entries.len() {
                let entry = &setlist.entries[index as usize];
                if state.songs.contains_key(&entry.song_id) {
                    found = Some(entry.id.clone());
                    break;
                }
                index += step;
            }
            let entry_id = found.ok_or_else(|| {
                if step < 0 {
                    "This is the first song in the setlist.".to_string()
                } else {
                    "This is the last song in the setlist.".to_string()
                }
            })?;
            (setlist_id, entry_id)
        };
        self.open_setlist_entry(&setlist_id, &entry_id)
    }

    /// Start a new unsaved song in the editor.
    ///
    /// # Errors
    ///
    /// Dirty editor already open, or persist failure reserving an id.
    pub fn create_song(&self) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        ensure_no_dirty_editor(&state)?;
        clear_setlist_session(&mut state);
        let id = SongId::new(format!("song-{}", state.next_id));
        state.next_id += 1;
        self.library
            .save_next_id(state.next_id)
            .map_err(|error| error.to_string())?;
        state.editor = Some(EditorSession::new_song(id));
        Ok(editor_view(
            state.editor.as_ref().expect("editor just created"),
        ))
    }

    /// Open the library song in the editor.
    ///
    /// # Errors
    ///
    /// Unknown id or dirty editor already open.
    pub fn begin_edit(&self, id: &str) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        ensure_no_dirty_editor(&state)?;
        clear_setlist_session(&mut state);
        let record = state
            .songs
            .get(id)
            .ok_or_else(|| format!("Song '{id}' was not found."))?
            .clone();
        state.session_id = Some(id.to_string());
        state.steps = steps_from_song(&record.song);
        let mut editor = EditorSession::from_library(record.song, record.tags, record.favorite);
        refresh_chord_warnings(&mut editor);
        state.editor = Some(editor);
        Ok(editor_view(
            state.editor.as_ref().expect("editor just opened"),
        ))
    }

    #[must_use]
    pub fn editor_state(&self) -> Option<EditorSessionView> {
        let state = self.lock();
        state.editor.as_ref().map(editor_view)
    }

    /// Commit the editor draft to the library.
    ///
    /// # Errors
    ///
    /// Editor closed, empty title, or persist failure.
    pub fn save_edit(&self) -> Result<EditorSaveResult, String> {
        let mut state = self.lock();
        let editor = state
            .editor
            .as_ref()
            .ok_or_else(|| "The editor is not open.".to_string())?;
        if editor.draft.title().trim().is_empty() {
            return Err("Title cannot be empty.".to_string());
        }
        let now = Timestamp::now();
        let mut song = editor.draft.clone();
        song.set_updated_at(Some(now));
        if song.created_at().is_none() {
            song.set_created_at(Some(now));
        }
        let song_id = song.id().as_str().to_string();
        let tags = editor.tags.clone();
        let favorite = editor.favorite;
        let warnings = editor.warnings.clone();
        let last_opened = state
            .songs
            .get(&song_id)
            .and_then(|record| record.last_opened_at)
            .or(Some(now.as_secs()));
        let record = StoredSong {
            song: song.clone(),
            favorite,
            tags,
            last_opened_at: last_opened,
            last_modified_at: Some(now.as_secs()),
        };
        self.persist_record(&record, None)?;
        state.songs.insert(song_id.clone(), record);
        state.session_id = Some(song_id);
        state.warnings = warnings;
        state.steps = steps_from_song(&song);
        if let Some(editor) = state.editor.as_mut() {
            editor.draft = song.clone();
            editor.baseline = Some(song);
            editor.is_new = false;
            editor.dirty = false;
        }
        Ok(EditorSaveResult {
            session: session_view(&state),
            editor: editor_view(state.editor.as_ref().expect("editor still open")),
        })
    }

    pub fn cancel_edit(&self) -> Option<SongSessionView> {
        let mut state = self.lock();
        state.editor = None;
        state.session_id.as_ref().map(|_| session_view(&state))
    }

    /// # Errors
    ///
    /// Editor closed, empty title, or invalid key/tempo/meter.
    pub fn editor_update_meta(
        &self,
        update: EditorMetaUpdate,
    ) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        apply_meta(&mut editor.draft, &update)?;
        editor.tags = library::normalize_tags(update.tags);
        editor.dirty = true;
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed or invalid section label.
    pub fn editor_add_section(
        &self,
        label: SectionLabelInput,
    ) -> Result<EditorSessionView, String> {
        let parsed = parse_label(&label)?;
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        editor
            .draft
            .sections_mut()
            .push(Section::new(parsed, vec![Line::lyrics("")]));
        editor.dirty = true;
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed, unknown section, or invalid label.
    pub fn editor_set_section_label(
        &self,
        index: usize,
        label: SectionLabelInput,
    ) -> Result<EditorSessionView, String> {
        let parsed = parse_label(&label)?;
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        let section = editor
            .draft
            .sections_mut()
            .get_mut(index)
            .ok_or_else(|| "That section was not found.".to_string())?;
        section.set_label(parsed);
        editor.dirty = true;
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed, unknown section, or last remaining section.
    pub fn editor_remove_section(&self, index: usize) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        if editor.draft.sections().len() <= 1 {
            return Err("A song needs at least one section.".to_string());
        }
        if index >= editor.draft.sections().len() {
            return Err("That section was not found.".to_string());
        }
        editor.draft.sections_mut().remove(index);
        editor.dirty = true;
        refresh_chord_warnings(editor);
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed or unknown section.
    pub fn editor_move_section(&self, from: usize, to: usize) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        let len = editor.draft.sections().len();
        if from >= len || to >= len {
            return Err("That section was not found.".to_string());
        }
        if from != to {
            let section = editor.draft.sections_mut().remove(from);
            editor.draft.sections_mut().insert(to, section);
            editor.dirty = true;
        }
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed or unknown section.
    pub fn editor_add_line(&self, section: usize) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        editor
            .draft
            .sections_mut()
            .get_mut(section)
            .ok_or_else(|| "That section was not found.".to_string())?
            .lines_mut()
            .push(Line::lyrics(""));
        editor.dirty = true;
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed, unknown line, or last remaining line.
    pub fn editor_remove_line(
        &self,
        section: usize,
        line: usize,
    ) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        let lines = editor
            .draft
            .sections_mut()
            .get_mut(section)
            .ok_or_else(|| "That section was not found.".to_string())?
            .lines_mut();
        if lines.len() <= 1 {
            return Err("A section needs at least one line.".to_string());
        }
        if line >= lines.len() {
            return Err("That line was not found.".to_string());
        }
        lines.remove(line);
        editor.dirty = true;
        refresh_chord_warnings(editor);
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed or unknown line.
    pub fn editor_set_lyrics(
        &self,
        section: usize,
        line: usize,
        lyrics: String,
    ) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        line_mut(&mut editor.draft, section, line)?.set_lyrics(lyrics);
        editor.dirty = true;
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed, unknown line, or empty symbol.
    pub fn editor_tag_chord(
        &self,
        section: usize,
        line: usize,
        lyric_index: u32,
        symbol: String,
    ) -> Result<EditorSessionView, String> {
        let chord = parse_chord_symbol(&symbol)?;
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        line_mut(&mut editor.draft, section, line)?.tag_chord(chord, lyric_index);
        editor.dirty = true;
        refresh_chord_warnings(editor);
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed or unknown chord.
    pub fn editor_untag_chord(
        &self,
        section: usize,
        line: usize,
        chord_index: usize,
    ) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        line_mut(&mut editor.draft, section, line)?.untag_chord(chord_index)?;
        editor.dirty = true;
        refresh_chord_warnings(editor);
        Ok(editor_view(editor))
    }

    /// Correct a tagged chord symbol (parser correction).
    ///
    /// # Errors
    ///
    /// Editor closed, unknown chord, or empty symbol.
    pub fn editor_replace_chord(
        &self,
        section: usize,
        line: usize,
        chord_index: usize,
        symbol: String,
    ) -> Result<EditorSessionView, String> {
        let chord = parse_chord_symbol(&symbol)?;
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        line_mut(&mut editor.draft, section, line)?.replace_chord(chord_index, chord)?;
        editor.dirty = true;
        refresh_chord_warnings(editor);
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed or unknown chord.
    pub fn editor_set_chord_index(
        &self,
        section: usize,
        line: usize,
        chord_index: usize,
        lyric_index: u32,
    ) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        line_mut(&mut editor.draft, section, line)?
            .set_chord_lyric_index(chord_index, lyric_index)?;
        editor.dirty = true;
        Ok(editor_view(editor))
    }

    /// # Errors
    ///
    /// Editor closed or unknown line.
    pub fn editor_set_annotation(
        &self,
        section: usize,
        line: usize,
        text: Option<String>,
    ) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        line_mut(&mut editor.draft, section, line)?.set_annotation(text);
        editor.dirty = true;
        Ok(editor_view(editor))
    }

    /// Replace the draft chart body by parsing pasted ChordPro or plain text.
    ///
    /// # Errors
    ///
    /// Editor closed or unknown import format.
    pub fn editor_parse_body(
        &self,
        text: &str,
        mode: ImportMode,
    ) -> Result<EditorSessionView, String> {
        let mut state = self.lock();
        let editor = editor_mut(&mut state)?;
        let id = SongId::new(editor.draft.id().as_str());
        let parsed = match mode {
            ImportMode::Auto => import_auto(text, id),
            ImportMode::ChordPro => import(text, ImportFormat::ChordPro, id),
            ImportMode::PlainText => import(text, ImportFormat::PlainText, id),
        };
        if editor.draft.title() == "Untitled" && parsed.song.title() != "Untitled" {
            editor.draft.set_title(parsed.song.title());
        }
        if editor.draft.artist().is_none() {
            if let Some(artist) = parsed.song.artist() {
                editor.draft.set_artist(Some(artist.to_string()));
            }
        }
        if editor.draft.original_key().is_none() {
            editor.draft.set_original_key(parsed.song.original_key());
            if editor.draft.performance_key().is_none() {
                editor
                    .draft
                    .set_performance_key(parsed.song.performance_key());
            }
        }
        if editor.draft.tempo().is_none() {
            editor.draft.set_tempo(parsed.song.tempo());
        }
        if editor.draft.time_signature().is_none() {
            editor
                .draft
                .set_time_signature(parsed.song.time_signature());
        }
        if editor.draft.notes().is_none() {
            if let Some(notes) = parsed.song.notes() {
                editor.draft.set_notes(Some(notes.to_string()));
            }
        }
        *editor.draft.sections_mut() = parsed.song.sections().to_vec();
        if editor.draft.sections().is_empty() {
            editor.draft.sections_mut().push(Section::new(
                tonic_domain::SectionLabel::Verse { number: None },
                vec![Line::lyrics("")],
            ));
        }
        editor.warnings = parsed.warnings;
        editor.dirty = true;
        Ok(editor_view(editor))
    }

    fn persist_setlist(
        &self,
        setlist: &StoredSetlist,
        ids: Option<(u64, u64)>,
    ) -> Result<(), String> {
        self.library
            .save_setlist(setlist)
            .map_err(|error| error.to_string())?;
        if let Some((next_setlist_id, next_entry_id)) = ids {
            self.library
                .save_next_setlist_ids(next_setlist_id, next_entry_id)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn persist_song_document(&self, state: &AppState) -> Result<(), String> {
        let id = state
            .session_id
            .as_deref()
            .ok_or_else(|| "No song is loaded.".to_string())?;
        let record = state
            .songs
            .get(id)
            .ok_or_else(|| "No song is loaded.".to_string())?;
        self.persist_record(record, None)
    }

    fn persist_open_setlist(&self, state: &mut AppState) -> Result<(), String> {
        let id = state
            .session_setlist_id
            .as_deref()
            .ok_or_else(|| "No setlist is open.".to_string())?
            .to_string();
        let setlist = state
            .setlists
            .get_mut(&id)
            .ok_or_else(|| "No setlist is open.".to_string())?;
        setlist.updated_at = Some(Timestamp::now().as_secs());
        let persisted = setlist.clone();
        self.persist_setlist(&persisted, None)
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
    let mut song = record.song.clone();
    let mut setlist_ctx = None;
    if let (Some(setlist_id), Some(entry_id)) = (
        state.session_setlist_id.as_deref(),
        state.session_entry_id.as_deref(),
    ) {
        if let Some(setlist) = state.setlists.get(setlist_id) {
            if let Some(entry) = setlist.entries.iter().find(|entry| entry.id == entry_id) {
                if let Some(symbol) = entry.performance_key.as_deref() {
                    if let Some(key) = Key::parse(symbol) {
                        song.set_performance_key(Some(key));
                    }
                }
                setlist_ctx = setlist::context(setlist, entry_id, song.performance_key());
            }
        }
    }
    SongSessionView::from_parts(
        &song,
        &state.warnings,
        state.steps,
        record.favorite,
        record.tags.clone(),
        setlist_ctx,
    )
}

fn clear_setlist_session(state: &mut AppState) {
    state.session_setlist_id = None;
    state.session_entry_id = None;
}

fn apply_setlist_steps(state: &mut AppState) -> Result<(), String> {
    let original = {
        let song = open_song_mut(state)?;
        song.original_key()
            .ok_or_else(|| "No original key.".to_string())?
    };
    let target = original.transpose_semitones(state.steps);
    set_open_entry_key(state, Some(target.symbol()))
}

fn set_open_entry_key(state: &mut AppState, key: Option<String>) -> Result<(), String> {
    let setlist_id = state
        .session_setlist_id
        .clone()
        .ok_or_else(|| "No setlist is open.".to_string())?;
    let entry_id = state
        .session_entry_id
        .clone()
        .ok_or_else(|| "No setlist entry is open.".to_string())?;
    let setlist = state
        .setlists
        .get_mut(&setlist_id)
        .ok_or_else(|| format!("Setlist '{setlist_id}' was not found."))?;
    let entry = setlist
        .entries
        .iter_mut()
        .find(|entry| entry.id == entry_id)
        .ok_or_else(|| format!("Setlist entry '{entry_id}' was not found."))?;
    entry.performance_key = key;
    Ok(())
}

fn entry_steps(song: &Song, setlist: &StoredSetlist, entry_id: &str) -> i32 {
    let Some(entry) = setlist.entries.iter().find(|entry| entry.id == entry_id) else {
        return steps_from_song(song);
    };
    match (
        song.original_key(),
        entry
            .performance_key
            .as_deref()
            .and_then(Key::parse)
            .or_else(|| song.performance_key()),
    ) {
        (Some(original), Some(performance)) => (i32::from(performance.pitch_class().value())
            - i32::from(original.pitch_class().value()))
        .rem_euclid(12),
        _ => 0,
    }
}

fn ensure_original_key_only(song: &mut Song) -> Key {
    if let Some(key) = song.original_key() {
        return key;
    }
    let inferred = infer_key(song);
    song.set_original_key(Some(inferred));
    inferred
}

fn ensure_no_dirty_editor(state: &AppState) -> Result<(), String> {
    if state.editor.as_ref().is_some_and(|editor| editor.dirty) {
        Err("Save or cancel the editor first.".to_string())
    } else {
        Ok(())
    }
}

fn editor_mut(state: &mut AppState) -> Result<&mut EditorSession, String> {
    state
        .editor
        .as_mut()
        .ok_or_else(|| "The editor is not open.".to_string())
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
        assert_eq!(info.phase, 9);
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
        services.close_song().unwrap();
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

    #[test]
    fn setlist_references_songs_and_keeps_independent_overrides() {
        let root = temp_root();
        {
            let services = AppServices::open(&root).unwrap();
            let _imported = services
                .import_text("{title: Grace}\n{key: C}\n[C]Hi", ImportMode::ChordPro)
                .unwrap();
            let setlist = services.create_setlist(Some("Gig".into())).unwrap();
            services.add_setlist_song(&setlist.id, "song-1").unwrap();
            services.add_setlist_song(&setlist.id, "song-1").unwrap();
            let detail = services.get_setlist(&setlist.id).unwrap();
            assert_eq!(detail.entries.len(), 2);
            assert_eq!(detail.entries[0].song_id, "song-1");
            assert_eq!(detail.entries[1].song_id, "song-1");
            assert_ne!(detail.entries[0].id, detail.entries[1].id);

            services
                .update_setlist_entry(
                    &setlist.id,
                    &detail.entries[0].id,
                    Some("Bb".into()),
                    Some(2),
                    Some("slow".into()),
                )
                .unwrap();
            let opened = services
                .open_setlist_entry(&setlist.id, &detail.entries[0].id)
                .unwrap();
            assert_eq!(opened.song.performance_key.as_deref(), Some("Bb"));
            assert_eq!(opened.setlist.as_ref().unwrap().capo_fret, Some(2));
            assert_eq!(
                opened.setlist.as_ref().unwrap().played_key.as_deref(),
                Some("Ab")
            );
            services.transpose_by(1).unwrap();
        }

        let services = AppServices::open(&root).unwrap();
        let list = services.list_setlists();
        assert_eq!(list.len(), 1);
        let detail = services.get_setlist(&list[0].id).unwrap();
        assert_eq!(detail.entries[0].performance_key.as_deref(), Some("B"));
        assert_eq!(detail.entries[0].capo_fret, Some(2));
        assert!(detail.entries[1].performance_key.is_none());
        let song = services.open_song("song-1").unwrap();
        assert_eq!(song.song.performance_key.as_deref(), Some("C"));
        assert!(song.setlist.is_none());

        let copy = services.duplicate_setlist(&list[0].id).unwrap();
        assert_eq!(copy.name, "Gig (copy)");
        assert_eq!(copy.entries.len(), 2);
        assert_ne!(copy.id, list[0].id);
        assert_ne!(copy.entries[0].id, detail.entries[0].id);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn setlist_neighbor_skips_missing_and_stops_at_ends() {
        let services = AppServices::in_memory();
        services
            .import_text("{title: One}\n{key: C}\n[C]Hi", ImportMode::ChordPro)
            .unwrap();
        services
            .import_text("{title: Two}\n{key: D}\n[D]Mid", ImportMode::ChordPro)
            .unwrap();
        services
            .import_text("{title: Three}\n{key: G}\n[G]Hey", ImportMode::ChordPro)
            .unwrap();
        let setlist = services.create_setlist(Some("Set".into())).unwrap();
        services.add_setlist_song(&setlist.id, "song-1").unwrap();
        services.add_setlist_song(&setlist.id, "song-2").unwrap();
        services.add_setlist_song(&setlist.id, "song-3").unwrap();
        services.delete_song("song-2").unwrap();
        let detail = services.get_setlist(&setlist.id).unwrap();
        assert_eq!(detail.entries.len(), 3);
        assert!(detail.entries[1].missing);

        let first = services
            .open_setlist_entry(&setlist.id, &detail.entries[0].id)
            .unwrap();
        assert_eq!(first.song.title, "One");
        assert_eq!(
            services.open_setlist_neighbor(-1).unwrap_err(),
            "This is the first song in the setlist."
        );
        let third = services.open_setlist_neighbor(1).unwrap();
        assert_eq!(third.song.title, "Three");
        assert_eq!(third.setlist.as_ref().unwrap().index, 2);
        assert_eq!(
            services.open_setlist_neighbor(1).unwrap_err(),
            "This is the last song in the setlist."
        );
        let back = services.open_setlist_neighbor(-1).unwrap();
        assert_eq!(back.song.title, "One");
    }

    #[test]
    fn create_save_and_reopen_manual_song() {
        let root = temp_root();
        {
            let services = AppServices::open(&root).unwrap();
            let created = services.create_song().unwrap();
            assert!(created.is_new);
            assert!(created.dirty);
            assert_eq!(created.title, "Untitled");
            assert!(services
                .list_library(LibraryQuery::default())
                .songs
                .is_empty());

            services
                .editor_update_meta(EditorMetaUpdate {
                    title: "New Tune".into(),
                    artist: Some("Me".into()),
                    album: None,
                    original_key: Some("G".into()),
                    tempo_bpm: Some(90),
                    time_signature: Some("4/4".into()),
                    notes: None,
                    tags: vec!["original".into()],
                })
                .unwrap();
            services
                .editor_set_lyrics(0, 0, "Hello world".into())
                .unwrap();
            let tagged = services.editor_tag_chord(0, 0, 0, "G".into()).unwrap();
            assert_eq!(tagged.sections[0].lines[0].chords[0].symbol, "G");
            assert_eq!(
                tagged.sections[0].lines[0].chords[0].status,
                "fullyRecognized"
            );

            services
                .editor_add_section(SectionLabelInput {
                    kind: "chorus".into(),
                    number: None,
                    custom_name: None,
                })
                .unwrap();
            services.editor_set_lyrics(1, 0, "Sing it".into()).unwrap();
            services.editor_tag_chord(1, 0, 0, "C".into()).unwrap();

            let saved = services.save_edit().unwrap();
            assert!(!saved.editor.dirty);
            assert!(!saved.editor.is_new);
            assert_eq!(saved.session.song.title, "New Tune");
            assert_eq!(saved.session.song.sections.len(), 2);
            assert_eq!(saved.session.tags, ["original"]);
        }

        let services = AppServices::open(&root).unwrap();
        let list = services.list_library(LibraryQuery::default());
        assert_eq!(list.songs.len(), 1);
        assert_eq!(list.songs[0].title, "New Tune");
        let opened = services.open_song(&list.songs[0].id).unwrap();
        assert_eq!(opened.song.sections[0].lines[0].lyrics, "Hello world");
        assert_eq!(opened.song.sections[0].lines[0].chords[0].written, "G");
        assert_eq!(opened.song.sections[1].label, "Chorus");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cancel_new_song_does_not_persist() {
        let services = AppServices::new();
        let _created = services.create_song().unwrap();
        services
            .editor_update_meta(EditorMetaUpdate {
                title: "Scratch".into(),
                artist: None,
                album: None,
                original_key: None,
                tempo_bpm: None,
                time_signature: None,
                notes: None,
                tags: vec![],
            })
            .unwrap();
        assert!(services.cancel_edit().is_none());
        assert!(services.editor_state().is_none());
        assert!(services
            .list_library(LibraryQuery::default())
            .songs
            .is_empty());
    }

    #[test]
    fn cancel_edit_restores_saved_song() {
        let services = AppServices::new();
        let _imported = services
            .import_text("{title: Keep}\n{key: C}\n[C]Hi", ImportMode::ChordPro)
            .unwrap();
        services.begin_edit("song-1").unwrap();
        services.editor_set_lyrics(0, 0, "Changed".into()).unwrap();
        assert!(services.editor_state().unwrap().dirty);
        let session = services.cancel_edit().unwrap();
        assert_eq!(session.song.sections[0].lines[0].lyrics, "Hi");
        assert!(services.editor_state().is_none());
    }

    #[test]
    fn parser_correction_and_parse_body() {
        let services = AppServices::new();
        let _imported = services
            .import_text("{title: Fix}\n[C]Hi [Xyz]there", ImportMode::ChordPro)
            .unwrap();
        let editing = services.begin_edit("song-1").unwrap();
        assert!(editing.sections[0].lines[0]
            .chords
            .iter()
            .any(|chord| chord.status == "unrecognized"));

        let corrected = services.editor_replace_chord(0, 0, 1, "G".into()).unwrap();
        assert_eq!(corrected.sections[0].lines[0].chords[1].symbol, "G");
        assert_eq!(
            corrected.sections[0].lines[0].chords[1].status,
            "fullyRecognized"
        );

        let parsed = services
            .editor_parse_body("[Am]Hello [E]world", ImportMode::ChordPro)
            .unwrap();
        assert_eq!(parsed.sections[0].lines[0].lyrics, "Hello world");
        assert_eq!(parsed.sections[0].lines[0].chords[0].symbol, "Am");
        assert_eq!(parsed.title, "Fix");
    }
}
