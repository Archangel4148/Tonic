//! Website URL import adapters.
//!
//! Each adapter extracts a chart from a supported site's HTML and converts it
//! into the canonical [`Song`] model. Fetching happens outside this module so
//! adapters stay unit-testable with fixtures.

mod ultimate_guitar;

use tonic_domain::SongId;

use crate::ImportResult;

pub use ultimate_guitar::{
    matches_ultimate_guitar_url, parse_ultimate_guitar_html, ULTIMATE_GUITAR_SITE,
};

/// Identifies a supported website adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebSite {
    UltimateGuitar,
}

impl WebSite {
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::UltimateGuitar => ULTIMATE_GUITAR_SITE,
        }
    }

    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::UltimateGuitar => "Ultimate Guitar",
        }
    }
}

/// Why a URL or page could not be imported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebImportError {
    UnsupportedUrl(String),
    ParseFailed(String),
}

impl std::fmt::Display for WebImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedUrl(message) | Self::ParseFailed(message) => f.write_str(message),
        }
    }
}

/// Match a user-pasted URL to a site adapter, if any.
#[must_use]
pub fn recognize_web_url(url: &str) -> Option<WebSite> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    if matches_ultimate_guitar_url(trimmed) {
        Some(WebSite::UltimateGuitar)
    } else {
        None
    }
}

/// Parse already-fetched HTML for `url` into a [`Song`].
///
/// # Errors
///
/// Unsupported host/URL shape, or page content that cannot be extracted.
pub fn import_web_html(
    url: &str,
    html: &str,
    id: impl Into<SongId>,
) -> Result<ImportResult, WebImportError> {
    let site = recognize_web_url(url).ok_or_else(|| {
        WebImportError::UnsupportedUrl(
            "That URL is not from a supported website yet. Ultimate Guitar chord tabs are supported."
                .to_string(),
        )
    })?;
    match site {
        WebSite::UltimateGuitar => parse_ultimate_guitar_html(url, html, id),
    }
}

/// Human-readable list of currently supported sites (for UI hints).
#[must_use]
pub fn supported_web_sites() -> Vec<&'static str> {
    vec![WebSite::UltimateGuitar.display_name()]
}
