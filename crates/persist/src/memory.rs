//! In-memory library used by unit tests.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{PersistError, SongLibrary, StoredSong};

/// Non-durable library that still implements the same boundary as the filesystem store.
pub struct MemoryLibrary {
    inner: Mutex<(u64, HashMap<String, StoredSong>)>,
}

impl MemoryLibrary {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new((1, HashMap::new())),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, (u64, HashMap<String, StoredSong>)> {
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
        let (next_id, songs) = &*self.lock();
        Ok((*next_id, songs.values().cloned().collect()))
    }

    fn save(&self, record: &StoredSong) -> Result<(), PersistError> {
        self.lock()
            .1
            .insert(record.song.id().as_str().to_string(), record.clone());
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), PersistError> {
        self.lock().1.remove(id);
        Ok(())
    }

    fn save_next_id(&self, next_id: u64) -> Result<(), PersistError> {
        self.lock().0 = next_id;
        Ok(())
    }
}
