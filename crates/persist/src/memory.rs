//! In-memory library used by unit tests.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{PersistError, SetlistSnapshot, SongLibrary, StoredSetlist, StoredSong};

struct MemoryInner {
    next_id: u64,
    next_setlist_id: u64,
    next_entry_id: u64,
    songs: HashMap<String, StoredSong>,
    setlists: HashMap<String, StoredSetlist>,
}

/// Non-durable library that still implements the same boundary as the filesystem store.
pub struct MemoryLibrary {
    inner: Mutex<MemoryInner>,
}

impl MemoryLibrary {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MemoryInner {
                next_id: 1,
                next_setlist_id: 1,
                next_entry_id: 1,
                songs: HashMap::new(),
                setlists: HashMap::new(),
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, MemoryInner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for MemoryLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl SongLibrary for MemoryLibrary {
    fn health_check(&self) -> Result<(), PersistError> {
        Ok(())
    }

    fn load_all(&self) -> Result<(u64, Vec<StoredSong>), PersistError> {
        let inner = self.lock();
        Ok((inner.next_id, inner.songs.values().cloned().collect()))
    }

    fn save(&self, record: &StoredSong) -> Result<(), PersistError> {
        self.lock()
            .songs
            .insert(record.song.id().as_str().to_string(), record.clone());
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), PersistError> {
        self.lock().songs.remove(id);
        Ok(())
    }

    fn save_next_id(&self, next_id: u64) -> Result<(), PersistError> {
        self.lock().next_id = next_id;
        Ok(())
    }

    fn load_setlists(&self) -> Result<SetlistSnapshot, PersistError> {
        let inner = self.lock();
        Ok(SetlistSnapshot {
            next_setlist_id: inner.next_setlist_id,
            next_entry_id: inner.next_entry_id,
            setlists: inner.setlists.values().cloned().collect(),
        })
    }

    fn save_setlist(&self, setlist: &StoredSetlist) -> Result<(), PersistError> {
        self.lock()
            .setlists
            .insert(setlist.id.clone(), setlist.clone());
        Ok(())
    }

    fn delete_setlist(&self, id: &str) -> Result<(), PersistError> {
        self.lock().setlists.remove(id);
        Ok(())
    }

    fn save_next_setlist_ids(
        &self,
        next_setlist_id: u64,
        next_entry_id: u64,
    ) -> Result<(), PersistError> {
        let mut inner = self.lock();
        inner.next_setlist_id = next_setlist_id;
        inner.next_entry_id = next_entry_id;
        Ok(())
    }
}
