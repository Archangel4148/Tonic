//! Persistence boundary for Tonic.
//!
//! Durable storage is introduced in Phase 6. This crate currently exposes a
//! tiny health-check API so the application layer can depend on a stable
//! persistence boundary without implementing song I/O yet.

use std::fmt;

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

/// Storage boundary used by application services.
pub trait Store: Send + Sync {
    /// Returns `Ok(())` when the backing store can accept work.
    ///
    /// # Errors
    ///
    /// Returns [`PersistError`] when the store cannot be used.
    fn health_check(&self) -> Result<(), PersistError>;
}

/// In-memory stub used until Phase 6 introduces durable storage.
#[derive(Debug, Default)]
pub struct MemoryStore;

impl MemoryStore {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Store for MemoryStore {
    fn health_check(&self) -> Result<(), PersistError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_is_healthy() {
        let store = MemoryStore::new();
        assert!(store.health_check().is_ok());
    }

    #[test]
    fn persist_error_preserves_message() {
        let error = PersistError::new("disk full");
        assert_eq!(error.message(), "disk full");
        assert_eq!(error.to_string(), "disk full");
    }
}
