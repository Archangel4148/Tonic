//! Durable local song library.
//!
//! Songs are stored as JSON files under a library root. Application services
//! keep the live copy in memory; this crate is the durable snapshot.

mod file;
mod memory;
mod record;
mod setlist;

use std::fmt;
use std::path::Path;

pub use file::{copy_library_tree, library_has_data, FileLibrary};
pub use memory::MemoryLibrary;
pub use record::{StoredSong, TransposeMode};
pub use setlist::{SetlistEntry, SetlistSnapshot, StoredSetlist};

/// Recoverable persistence failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistError {
    message: String,
}

impl PersistError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for PersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PersistError {}

/// Song library storage boundary.
pub trait SongLibrary: Send + Sync {
    fn health_check(&self) -> Result<(), PersistError>;
    fn load_all(&self) -> Result<(u64, Vec<StoredSong>), PersistError>;
    fn save(&self, record: &StoredSong) -> Result<(), PersistError>;
    fn delete(&self, id: &str) -> Result<(), PersistError>;
    fn save_next_id(&self, next_id: u64) -> Result<(), PersistError>;
    fn load_setlists(&self) -> Result<SetlistSnapshot, PersistError>;
    fn save_setlist(&self, setlist: &StoredSetlist) -> Result<(), PersistError>;
    fn delete_setlist(&self, id: &str) -> Result<(), PersistError>;
    fn save_next_setlist_ids(
        &self,
        next_setlist_id: u64,
        next_entry_id: u64,
    ) -> Result<(), PersistError>;
}

/// Open a filesystem library, creating the directory if needed.
///
/// # Errors
///
/// Returns [`PersistError`] when the directory cannot be created or read.
pub fn open_file_library(root: impl AsRef<Path>) -> Result<FileLibrary, PersistError> {
    FileLibrary::open(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic_domain::{
        parse_chord, ChordToken, Line, LineToken, LyricToken, Section, SectionLabel, Song,
    };

    fn demo_song(id: &str, title: &str) -> Song {
        let line = Line::new(vec![
            LineToken::Chord(ChordToken::new(parse_chord("C"))),
            LineToken::Lyric(LyricToken::new("Hello")),
        ]);
        Song::builder(id, title)
            .section(Section::new(
                SectionLabel::Verse { number: None },
                vec![line],
            ))
            .build()
    }

    #[test]
    fn persist_error_preserves_message() {
        let error = PersistError::new("disk full");
        assert_eq!(error.message(), "disk full");
        assert_eq!(error.to_string(), "disk full");
    }

    #[test]
    fn file_library_round_trips_and_deletes() {
        let root = std::env::temp_dir().join(format!(
            "tonic-persist-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let library = FileLibrary::open(&root).unwrap();
        assert!(library.health_check().is_ok());

        let record = StoredSong {
            song: demo_song("song-1", "Demo"),
            favorite: true,
            tags: vec!["gospel".into()],
            last_opened_at: Some(10),
            last_modified_at: Some(20),
            transpose_mode: crate::TransposeMode::Chords,
            capo_fret: None,
            shapes_key: None,
        };
        library.save(&record).unwrap();
        library.save_next_id(2).unwrap();

        let (next_id, loaded) = library.load_all().unwrap();
        assert_eq!(next_id, 2);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].song.title(), "Demo");
        assert!(loaded[0].favorite);
        assert_eq!(loaded[0].tags, ["gospel"]);

        library.delete("song-1").unwrap();
        let (_, remaining) = library.load_all().unwrap();
        assert!(remaining.is_empty());

        let setlist = StoredSetlist {
            id: "setlist-1".into(),
            name: "Gig".into(),
            notes: Some("Friday".into()),
            event_date: Some("2026-08-14".into()),
            entries: vec![SetlistEntry {
                id: "entry-1".into(),
                song_id: "song-1".into(),
                performance_key: Some("Bb".into()),
                capo_fret: Some(2),
                notes: Some("slow intro".into()),
                transpose_mode: crate::TransposeMode::Chords,
                shapes_key: None,
            }],
            updated_at: Some(30),
        };
        library.save_setlist(&setlist).unwrap();
        library.save_next_setlist_ids(2, 2).unwrap();
        let snapshot = library.load_setlists().unwrap();
        assert_eq!(snapshot.next_setlist_id, 2);
        assert_eq!(snapshot.next_entry_id, 2);
        assert_eq!(snapshot.setlists.len(), 1);
        assert_eq!(snapshot.setlists[0].entries[0].song_id, "song-1");
        library.delete_setlist("setlist-1").unwrap();
        assert!(library.load_setlists().unwrap().setlists.is_empty());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn copy_library_tree_duplicates_json_files() {
        let root = std::env::temp_dir().join(format!(
            "tonic-persist-copy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let library = FileLibrary::open(&root).unwrap();
        library
            .save(&StoredSong {
                song: demo_song("song-1", "Demo"),
                favorite: false,
                tags: vec![],
                last_opened_at: None,
                last_modified_at: None,
                transpose_mode: crate::TransposeMode::Chords,
                capo_fret: None,
                shapes_key: None,
            })
            .unwrap();
        library.save_next_id(2).unwrap();
        assert!(library_has_data(&root));

        let dest = root.join("visible-copy");
        copy_library_tree(&root, &dest).unwrap();
        assert!(library_has_data(&dest));
        assert!(dest.join("index.json").is_file());
        assert!(dest.join("songs").join("song-1.json").is_file());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn memory_library_is_isolated() {
        let library = MemoryLibrary::new();
        library
            .save(&StoredSong {
                song: demo_song("a", "A"),
                favorite: false,
                tags: vec![],
                last_opened_at: None,
                last_modified_at: None,
                transpose_mode: crate::TransposeMode::Chords,
                capo_fret: None,
                shapes_key: None,
            })
            .unwrap();
        let (_, songs) = library.load_all().unwrap();
        assert_eq!(songs.len(), 1);
        library.delete("a").unwrap();
        assert!(library.load_all().unwrap().1.is_empty());
    }
}
