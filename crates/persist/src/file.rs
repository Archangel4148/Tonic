//! JSON files under a library root: `index.json`, `songs/{id}.json`, `setlists/{id}.json`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{PersistError, SetlistSnapshot, SongLibrary, StoredSetlist, StoredSong};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LibraryIndex {
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    next_setlist_id: u64,
    #[serde(default)]
    next_entry_id: u64,
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
        fs::create_dir_all(root.join("setlists")).map_err(|error| {
            PersistError::new(format!("Could not create setlists directory: {error}"))
        })?;
        Ok(Self { root })
    }

    fn index_path(&self) -> PathBuf {
        self.root.join("index.json")
    }

    fn song_path(&self, id: &str) -> Result<PathBuf, PersistError> {
        Ok(self
            .root
            .join("songs")
            .join(format!("{}.json", safe_id(id)?)))
    }

    fn setlist_path(&self, id: &str) -> Result<PathBuf, PersistError> {
        Ok(self
            .root
            .join("setlists")
            .join(format!("{}.json", safe_id(id)?)))
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

    fn write_index(&self, index: &LibraryIndex) -> Result<(), PersistError> {
        let json = serde_json::to_string_pretty(index)
            .map_err(|error| PersistError::new(format!("Could not serialize index: {error}")))?;
        fs::write(self.index_path(), json)
            .map_err(|error| PersistError::new(format!("Could not write library index: {error}")))
    }

    fn read_json_dir<T: serde::de::DeserializeOwned>(
        &self,
        dir: &Path,
        kind: &str,
    ) -> Result<Vec<T>, PersistError> {
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut records = Vec::new();
        let entries = fs::read_dir(dir).map_err(|error| {
            PersistError::new(format!("Could not read {kind} directory: {error}"))
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                PersistError::new(format!("Could not read {kind} directory: {error}"))
            })?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|error| {
                PersistError::new(format!("Could not read {}: {error}", path.display()))
            })?;
            let record: T = serde_json::from_str(&text).map_err(|error| {
                PersistError::new(format!("Corrupt {kind} file {}: {error}", path.display()))
            })?;
            records.push(record);
        }
        Ok(records)
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
        let records = self.read_json_dir::<StoredSong>(&self.root.join("songs"), "songs")?;
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
        let mut index = self.read_index()?;
        index.next_id = next_id;
        self.write_index(&index)
    }

    fn load_setlists(&self) -> Result<SetlistSnapshot, PersistError> {
        let index = self.read_index()?;
        let setlists =
            self.read_json_dir::<StoredSetlist>(&self.root.join("setlists"), "setlists")?;
        let mut next_setlist_id = index.next_setlist_id.max(1);
        let mut next_entry_id = index.next_entry_id.max(1);
        for setlist in &setlists {
            if let Some(number) = setlist
                .id
                .strip_prefix("setlist-")
                .and_then(|value| value.parse::<u64>().ok())
            {
                next_setlist_id = next_setlist_id.max(number + 1);
            }
            for entry in &setlist.entries {
                if let Some(number) = entry
                    .id
                    .strip_prefix("entry-")
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    next_entry_id = next_entry_id.max(number + 1);
                }
            }
        }
        Ok(SetlistSnapshot {
            next_setlist_id,
            next_entry_id,
            setlists,
        })
    }

    fn save_setlist(&self, setlist: &StoredSetlist) -> Result<(), PersistError> {
        let path = self.setlist_path(&setlist.id)?;
        let json = serde_json::to_string_pretty(setlist)
            .map_err(|error| PersistError::new(format!("Could not serialize setlist: {error}")))?;
        fs::write(&path, json)
            .map_err(|error| PersistError::new(format!("Could not write setlist: {error}")))
    }

    fn delete_setlist(&self, id: &str) -> Result<(), PersistError> {
        let path = self.setlist_path(id)?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(PersistError::new(format!(
                "Could not delete setlist: {error}"
            ))),
        }
    }

    fn save_next_setlist_ids(
        &self,
        next_setlist_id: u64,
        next_entry_id: u64,
    ) -> Result<(), PersistError> {
        let mut index = self.read_index()?;
        index.next_setlist_id = next_setlist_id;
        index.next_entry_id = next_entry_id;
        self.write_index(&index)
    }
}

fn safe_id(id: &str) -> Result<&str, PersistError> {
    if id.is_empty()
        || !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(PersistError::new("Invalid id for storage."));
    }
    Ok(id)
}

/// True when `root` already has a library index or song JSON files.
#[must_use]
pub fn library_has_data(root: &Path) -> bool {
    if root.join("index.json").is_file() {
        return true;
    }
    let songs = root.join("songs");
    let Ok(entries) = fs::read_dir(songs) else {
        return false;
    };
    entries
        .filter_map(Result::ok)
        .any(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
}

/// Copy `index.json`, `songs/`, and `setlists/` from `from` into `to`.
///
/// # Errors
///
/// Returns [`PersistError`] when directories or files cannot be copied.
pub fn copy_library_tree(from: &Path, to: &Path) -> Result<(), PersistError> {
    if from == to {
        return Ok(());
    }
    if !from.exists() {
        return Err(PersistError::new(format!(
            "Nothing to copy from {}.",
            from.display()
        )));
    }
    fs::create_dir_all(to).map_err(|error| {
        PersistError::new(format!(
            "Could not create save folder {}: {error}",
            to.display()
        ))
    })?;
    copy_if_present(&from.join("index.json"), &to.join("index.json"))?;
    copy_dir_json(&from.join("songs"), &to.join("songs"))?;
    copy_dir_json(&from.join("setlists"), &to.join("setlists"))?;
    Ok(())
}

fn copy_if_present(from: &Path, to: &Path) -> Result<(), PersistError> {
    if !from.is_file() {
        return Ok(());
    }
    fs::copy(from, to).map_err(|error| {
        PersistError::new(format!(
            "Could not copy {} to {}: {error}",
            from.display(),
            to.display()
        ))
    })?;
    Ok(())
}

fn copy_dir_json(from: &Path, to: &Path) -> Result<(), PersistError> {
    if !from.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(to).map_err(|error| {
        PersistError::new(format!("Could not create {}: {error}", to.display()))
    })?;
    let entries = fs::read_dir(from).map_err(|error| {
        PersistError::new(format!("Could not read {}: {error}", from.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            PersistError::new(format!("Could not read {}: {error}", from.display()))
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        let dest = to.join(name);
        fs::copy(&path, &dest).map_err(|error| {
            PersistError::new(format!(
                "Could not copy {} to {}: {error}",
                path.display(),
                dest.display()
            ))
        })?;
    }
    Ok(())
}
