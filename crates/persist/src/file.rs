//! JSON files under a library root: `index.json` + `songs/{id}.json`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{PersistError, SongLibrary, StoredSong};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LibraryIndex {
    #[serde(default)]
    next_id: u64,
}

/// Filesystem song library.
pub struct FileLibrary {
    root: PathBuf,
}

impl FileLibrary {
    /// Create or open a library directory.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError`] when the directory cannot be created.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, PersistError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join("songs")).map_err(|error| {
            PersistError::new(format!("Could not create library directory: {error}"))
        })?;
        Ok(Self { root })
    }

    fn index_path(&self) -> PathBuf {
        self.root.join("index.json")
    }

    fn song_path(&self, id: &str) -> Result<PathBuf, PersistError> {
        if !is_safe_id(id) {
            return Err(PersistError::new("Invalid song id for storage."));
        }
        Ok(self.root.join("songs").join(format!("{id}.json")))
    }

    fn read_index(&self) -> Result<LibraryIndex, PersistError> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(LibraryIndex::default());
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| PersistError::new(format!("Could not read library index: {error}")))?;
        serde_json::from_str(&text)
            .map_err(|error| PersistError::new(format!("Corrupt library index: {error}")))
    }
}

impl SongLibrary for FileLibrary {
    fn health_check(&self) -> Result<(), PersistError> {
        let probe = self.root.join(".write-check");
        fs::write(&probe, b"ok")
            .and_then(|()| fs::remove_file(&probe))
            .map_err(|error| PersistError::new(format!("Library is not writable: {error}")))
    }

    fn load_all(&self) -> Result<(u64, Vec<StoredSong>), PersistError> {
        let index = self.read_index()?;
        let songs_dir = self.root.join("songs");
        let mut records = Vec::new();
        let entries = fs::read_dir(&songs_dir).map_err(|error| {
            PersistError::new(format!("Could not read songs directory: {error}"))
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                PersistError::new(format!("Could not read songs directory: {error}"))
            })?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|error| {
                PersistError::new(format!("Could not read {}: {error}", path.display()))
            })?;
            let record: StoredSong = serde_json::from_str(&text).map_err(|error| {
                PersistError::new(format!("Corrupt song file {}: {error}", path.display()))
            })?;
            records.push(record);
        }

        let mut next_id = index.next_id.max(1);
        for record in &records {
            if let Some(number) = record
                .song
                .id()
                .as_str()
                .strip_prefix("song-")
                .and_then(|value| value.parse::<u64>().ok())
            {
                next_id = next_id.max(number + 1);
            }
        }
        Ok((next_id, records))
    }

    fn save(&self, record: &StoredSong) -> Result<(), PersistError> {
        let path = self.song_path(record.song.id().as_str())?;
        let json = serde_json::to_string_pretty(record)
            .map_err(|error| PersistError::new(format!("Could not serialize song: {error}")))?;
        fs::write(&path, json)
            .map_err(|error| PersistError::new(format!("Could not write song: {error}")))
    }

    fn delete(&self, id: &str) -> Result<(), PersistError> {
        let path = self.song_path(id)?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(PersistError::new(format!("Could not delete song: {error}"))),
        }
    }

    fn save_next_id(&self, next_id: u64) -> Result<(), PersistError> {
        let index = LibraryIndex { next_id };
        let json = serde_json::to_string_pretty(&index)
            .map_err(|error| PersistError::new(format!("Could not serialize index: {error}")))?;
        fs::write(self.index_path(), json)
            .map_err(|error| PersistError::new(format!("Could not write library index: {error}")))
    }
}

fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}
