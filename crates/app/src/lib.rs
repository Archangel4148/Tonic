//! Application services and authoritative in-memory state ownership.
//!
//! The UI must not own domain data. Persistence is not the source of truth
//! for the running session. This crate orchestrates domain, import, and
//! persistence without depending on Tauri or React.

mod view;

use std::sync::Mutex;

use tonic_domain::{engine_name, engine_version, Key, ParseStatus, Quality, Song, SongId};
use tonic_import::{import, import_auto, ImportFormat, ImportWarning};
use tonic_persist::{MemoryStore, Store};

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

struct Session {
    song: Option<Song>,
    warnings: Vec<ImportWarning>,
    steps: i32,
    next_id: u64,
}

/// In-process application services.
///
/// Phase 5 owns the current song in memory and exposes import + transpose.
/// Durable library storage is Phase 6.
pub struct AppServices {
    store: MemoryStore,
    session: Mutex<Session>,
}

impl AppServices {
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: MemoryStore::new(),
            session: Mutex::new(Session {
                song: None,
                warnings: Vec::new(),
                steps: 0,
                next_id: 1,
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Session> {
        self.session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[must_use]
    pub fn info(&self) -> AppInfo {
        AppInfo {
            name: "Tonic",
            version: env!("CARGO_PKG_VERSION"),
            phase: 5,
            domain_engine: engine_name(),
            domain_version: engine_version(),
        }
    }

    #[must_use]
    pub fn persistence_healthy(&self) -> bool {
        self.store.health_check().is_ok()
    }

    /// Import a chord sheet, replace the session song, and return a view DTO.
    #[must_use]
    pub fn import_text(&self, input: &str, mode: ImportMode) -> SongSessionView {
        let mut session = self.lock();
        let id = SongId::new(format!("session-{}", session.next_id));
        session.next_id += 1;
        let result = match mode {
            ImportMode::Auto => import_auto(input, id),
            ImportMode::ChordPro => import(input, ImportFormat::ChordPro, id),
            ImportMode::PlainText => import(input, ImportFormat::PlainText, id),
        };
        session.warnings = result.warnings;
        session.steps = 0;
        session.song = Some(result.song);
        session_view(&session)
    }

    #[must_use]
    pub fn current_session(&self) -> Option<SongSessionView> {
        let session = self.lock();
        session.song.is_some().then(|| session_view(&session))
    }

    /// Shift performance key by `semitones`. Infers original key if missing.
    ///
    /// # Errors
    ///
    /// No song loaded.
    pub fn transpose_by(&self, semitones: i32) -> Result<SongSessionView, String> {
        let mut session = self.lock();
        {
            let song = session.song.as_mut().ok_or("No song is loaded.")?;
            ensure_original_key(song);
        }
        session.steps += semitones;
        apply_steps(&mut session);
        Ok(session_view(&session))
    }

    /// Set the performance key by symbol (`D`, `F#m`, …).
    ///
    /// # Errors
    ///
    /// No song loaded, or the symbol is not a valid key.
    pub fn set_performance_key(&self, symbol: &str) -> Result<SongSessionView, String> {
        let target = Key::parse(symbol).ok_or_else(|| format!("Unknown key '{symbol}'."))?;
        let mut session = self.lock();
        let original = {
            let song = session.song.as_mut().ok_or("No song is loaded.")?;
            ensure_original_key(song)
        };
        let diff =
            i32::from(target.pitch_class().value()) - i32::from(original.pitch_class().value());
        session.steps = diff.rem_euclid(12);
        if let Some(song) = session.song.as_mut() {
            song.set_performance_key(Some(target));
        }
        Ok(session_view(&session))
    }

    /// Restore performance key to the original key.
    ///
    /// # Errors
    ///
    /// No song loaded.
    pub fn reset_performance_key(&self) -> Result<SongSessionView, String> {
        let mut session = self.lock();
        {
            let song = session.song.as_mut().ok_or("No song is loaded.")?;
            if let Some(original) = song.original_key() {
                song.set_performance_key(Some(original));
            }
        }
        session.steps = 0;
        Ok(session_view(&session))
    }

    pub fn clear_song(&self) {
        let mut session = self.lock();
        session.song = None;
        session.warnings.clear();
        session.steps = 0;
    }
}

impl Default for AppServices {
    fn default() -> Self {
        Self::new()
    }
}

fn session_view(session: &Session) -> SongSessionView {
    let song = session.song.as_ref().expect("session view requires a song");
    SongSessionView::from_parts(song, &session.warnings, session.steps)
}

fn apply_steps(session: &mut Session) {
    let Some(song) = session.song.as_mut() else {
        return;
    };
    let Some(original) = song.original_key() else {
        return;
    };
    song.set_performance_key(Some(original.transpose_semitones(session.steps)));
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

#[cfg(test)]
mod tests {
    use super::*;
    use tonic_import::UNRECOGNIZED_CONTENT_MESSAGE;

    #[test]
    fn services_report_healthy_stack() {
        let services = AppServices::new();
        let info = services.info();

        assert_eq!(info.name, "Tonic");
        assert_eq!(info.phase, 5);
        assert_eq!(info.domain_engine, "tonic-domain");
        assert!(!info.version.is_empty());
        assert!(!info.domain_version.is_empty());
        assert!(services.persistence_healthy());
    }

    #[test]
    fn import_stores_session_and_builds_view() {
        let services = AppServices::new();
        let session = services.import_text(
            "{title: Demo}\n{key: C}\n[C]Hi [G]there",
            ImportMode::ChordPro,
        );
        assert_eq!(session.song.title, "Demo");
        assert_eq!(session.song.original_key.as_deref(), Some("C"));
        assert_eq!(session.song.performance_key.as_deref(), Some("C"));
        assert_eq!(session.song.sections[0].lines[0].lyrics, "Hi there");
        assert_eq!(session.song.sections[0].lines[0].chords[0].symbol, "C");
        assert_eq!(services.current_session().unwrap().song.id, session.song.id);
    }

    #[test]
    fn transpose_updates_display_not_written_chords_or_source() {
        let services = AppServices::new();
        let _imported = services.import_text(
            "{title: Demo}\n{key: C}\n[C]Hi [G]there",
            ImportMode::ChordPro,
        );
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
        let imported = services.import_text("[Am]Hello [E]world", ImportMode::ChordPro);
        assert!(imported.song.original_key.is_none());

        let up = services.transpose_by(2).unwrap();
        assert_eq!(up.song.original_key.as_deref(), Some("Am"));
        assert_eq!(up.song.performance_key.as_deref(), Some("Bm"));
        assert_eq!(up.song.sections[0].lines[0].chords[0].symbol, "Bm");
        assert_eq!(up.song.sections[0].lines[0].chords[0].written, "Am");
    }

    #[test]
    fn set_performance_key_and_clear() {
        let services = AppServices::new();
        let _imported = services.import_text("{title: X}\n{key: G}\n[G]Hi", ImportMode::ChordPro);
        let session = services.set_performance_key("A").unwrap();
        assert_eq!(session.song.performance_key.as_deref(), Some("A"));
        assert_eq!(session.song.sections[0].lines[0].chords[0].symbol, "A");
        services.clear_song();
        assert!(services.current_session().is_none());
        assert_eq!(services.transpose_by(1).unwrap_err(), "No song is loaded.");
    }

    #[test]
    fn import_warnings_surface_summary() {
        let services = AppServices::new();
        let session = services.import_text("{title: X}\n[C]Hi [Xyz]there", ImportMode::ChordPro);
        assert!(session.summary_message.as_deref() == Some(UNRECOGNIZED_CONTENT_MESSAGE));
        assert!(!session.warnings.is_empty());
        assert_eq!(
            session.song.sections[0].lines[0].chords[1].status,
            "unrecognized"
        );
        assert_eq!(session.song.sections[0].lines[0].chords[1].symbol, "Xyz");
    }
}
