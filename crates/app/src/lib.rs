//! Application services and authoritative in-memory state ownership.
//!
//! The UI must not own domain data. Persistence is not the source of truth
//! for the running session. This crate orchestrates domain and persistence
//! without depending on Tauri or React.

use tonic_domain::{engine_name, engine_version};
use tonic_persist::{MemoryStore, Store};

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
/// Phase 1 only exposes health and identity. Song library, setlists, and
/// preferences will live here in later phases.
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
            phase: 1,
            domain_engine: engine_name(),
            domain_version: engine_version(),
        }
    }

    #[must_use]
    pub fn persistence_healthy(&self) -> bool {
        self.store.health_check().is_ok()
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
        assert_eq!(info.phase, 1);
        assert_eq!(info.domain_engine, "tonic-domain");
        assert!(!info.version.is_empty());
        assert!(!info.domain_version.is_empty());
        assert!(services.persistence_healthy());
    }
}
