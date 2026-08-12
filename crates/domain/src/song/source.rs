//! Original import source. Changing keys must not destroy this.

use serde::{Deserialize, Serialize};

/// Where a song document came from.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceFormat {
    ChordPro,
    PlainText,
    MusicXml,
    Web,
    #[default]
    Manual,
    Other(String),
}

/// Preserved import metadata. Not the authoritative song body.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSource {
    format: SourceFormat,
    #[serde(default)]
    original_content: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    website: Option<String>,
}

impl Default for SongSource {
    fn default() -> Self {
        Self::manual()
    }
}

impl SongSource {
    #[must_use]
    pub fn manual() -> Self {
        Self {
            format: SourceFormat::Manual,
            original_content: None,
            url: None,
            website: None,
        }
    }

    #[must_use]
    pub fn chordpro(original_content: impl Into<String>) -> Self {
        Self {
            format: SourceFormat::ChordPro,
            original_content: Some(original_content.into()),
            url: None,
            website: None,
        }
    }

    #[must_use]
    pub fn plain_text(original_content: impl Into<String>) -> Self {
        Self {
            format: SourceFormat::PlainText,
            original_content: Some(original_content.into()),
            url: None,
            website: None,
        }
    }

    #[must_use]
    pub fn music_xml(original_content: impl Into<String>) -> Self {
        Self {
            format: SourceFormat::MusicXml,
            original_content: Some(original_content.into()),
            url: None,
            website: None,
        }
    }

    #[must_use]
    pub fn web(
        url: impl Into<String>,
        website: impl Into<String>,
        original_content: Option<String>,
    ) -> Self {
        Self {
            format: SourceFormat::Web,
            original_content,
            url: Some(url.into()),
            website: Some(website.into()),
        }
    }

    #[must_use]
    pub fn format(&self) -> &SourceFormat {
        &self.format
    }

    #[must_use]
    pub fn original_content(&self) -> Option<&str> {
        self.original_content.as_deref()
    }

    #[must_use]
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    #[must_use]
    pub fn website(&self) -> Option<&str> {
        self.website.as_deref()
    }
}
