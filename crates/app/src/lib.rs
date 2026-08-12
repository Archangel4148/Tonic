//! Application services and authoritative in-memory state ownership.
//!
//! The UI must not own domain data. Persistence is not the source of truth
//! for the running session. This crate orchestrates domain, import, and
//! persistence without depending on Tauri or React.

use tonic_domain::{engine_name, engine_version, SongId};
use tonic_persist::{MemoryStore, Store};

pub use tonic_import::{
    import, ImportFormat, ImportResult, ImportWarning, WarningKind, UNRECOGNIZED_CONTENT_MESSAGE,
};

/// Snapshot of process-level application identity and engine status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub phase: u32,
    pub domain_engine: &'static str,
    pub domain_version: &'static str,
}

/// In-process application services.
///
/// Phase 4 exposes import through application services. IPC/UI for import
/// arrives in Phase 5. Library persistence is Phase 6.
#[derive(Debug)]
pub struct AppServices {
    store: MemoryStore,
}

impl AppServices {
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: MemoryStore::new(),
        }
    }

    #[must_use]
    pub fn info(&self) -> AppInfo {
        AppInfo {
            name: "Tonic",
            version: env!("CARGO_PKG_VERSION"),
            phase: 4,
            domain_engine: engine_name(),
            domain_version: engine_version(),
        }
    }

    #[must_use]
    pub fn persistence_healthy(&self) -> bool {
        self.store.health_check().is_ok()
    }

    /// Import a chord sheet into the canonical song model.
    #[must_use]
    pub fn import_song(
        &self,
        input: &str,
        format: ImportFormat,
        id: impl Into<SongId>,
    ) -> ImportResult {
        import(input, format, id)
    }
}

impl Default for AppServices {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn services_report_healthy_stack() {
        let services = AppServices::new();
        let info = services.info();

        assert_eq!(info.name, "Tonic");
        assert_eq!(info.phase, 4);
        assert_eq!(info.domain_engine, "tonic-domain");
        assert!(!info.version.is_empty());
        assert!(!info.domain_version.is_empty());
        assert!(services.persistence_healthy());
    }

    #[test]
    fn import_song_uses_tonic_import() {
        let services = AppServices::new();
        let result = services.import_song("{title: Demo}\n[C]Hi", ImportFormat::ChordPro, "demo");
        assert_eq!(result.song.title(), "Demo");
        assert_eq!(result.song.sections()[0].lines()[0].lyric_text(), "Hi");
    }
}
