//! Semantic song sections. Custom names are always allowed.

use serde::{Deserialize, Serialize};

use super::line::Line;

/// One labeled block of lines.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    label: SectionLabel,
    #[serde(default)]
    lines: Vec<Line>,
}

/// Predefined kinds plus an open-ended custom label.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SectionLabel {
    Intro,
    Verse { number: Option<u16> },
    PreChorus,
    Chorus { number: Option<u16> },
    Bridge,
    Instrumental,
    Solo,
    Outro,
    Custom { name: String },
}

impl Section {
    #[must_use]
    pub fn new(label: SectionLabel, lines: Vec<Line>) -> Self {
        Self { label, lines }
    }

    #[must_use]
    pub fn label(&self) -> &SectionLabel {
        &self.label
    }

    #[must_use]
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn lines_mut(&mut self) -> &mut Vec<Line> {
        &mut self.lines
    }
}

impl SectionLabel {
    #[must_use]
    pub fn display_name(&self) -> String {
        match self {
            Self::Intro => "Intro".to_string(),
            Self::Verse { number: Some(n) } => format!("Verse {n}"),
            Self::Verse { number: None } => "Verse".to_string(),
            Self::PreChorus => "Pre-Chorus".to_string(),
            Self::Chorus { number: Some(n) } => format!("Chorus {n}"),
            Self::Chorus { number: None } => "Chorus".to_string(),
            Self::Bridge => "Bridge".to_string(),
            Self::Instrumental => "Instrumental".to_string(),
            Self::Solo => "Solo".to_string(),
            Self::Outro => "Outro".to_string(),
            Self::Custom { name } => name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_and_numbered_labels() {
        assert_eq!(
            SectionLabel::Verse { number: Some(2) }.display_name(),
            "Verse 2"
        );
        assert_eq!(
            SectionLabel::Custom {
                name: "Breakdown".into(),
            }
            .display_name(),
            "Breakdown"
        );
        assert_eq!(SectionLabel::PreChorus.display_name(), "Pre-Chorus");
    }
}
