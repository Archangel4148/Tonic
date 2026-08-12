//! Music-theory and song-domain layer for Tonic.
//!
//! This crate must remain independent of UI, Tauri, and persistence.
//! Phase 1 only establishes the boundary; chord parsing and transposition
//! arrive in Phase 2.

/// Current product phase implemented by this crate's public surface.
pub const PHASE: u32 = 1;

/// Human-readable identity of the domain engine.
#[must_use]
pub fn engine_name() -> &'static str {
    "tonic-domain"
}

/// Semantic version of the domain crate, taken from Cargo.toml.
#[must_use]
pub fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Confirms the domain layer can execute without UI dependencies.
#[must_use]
pub fn is_available() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_is_available_without_ui() {
        assert!(is_available());
        assert_eq!(engine_name(), "tonic-domain");
        assert!(!engine_version().is_empty());
        assert_eq!(PHASE, 1);
    }
}
