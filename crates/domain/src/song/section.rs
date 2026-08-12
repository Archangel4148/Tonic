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

    pub fn set_label(&mut self, label: SectionLabel) {
        self.label = label;
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

    #[must_use]
    pub fn kind_key(&self) -> &'static str {
        match self {
            Self::Intro => "intro",
            Self::Verse { .. } => "verse",
            Self::PreChorus => "preChorus",
            Self::Chorus { .. } => "chorus",
            Self::Bridge => "bridge",
            Self::Instrumental => "instrumental",
            Self::Solo => "solo",
            Self::Outro => "outro",
            Self::Custom { .. } => "custom",
        }
    }

    #[must_use]
    pub fn number(&self) -> Option<u16> {
        match self {
            Self::Verse { number } | Self::Chorus { number } => *number,
            _ => None,
        }
    }

    #[must_use]
    pub fn custom_name(&self) -> Option<&str> {
        match self {
            Self::Custom { name } => Some(name.as_str()),
            _ => None,
        }
    }

    /// Parse UI/IPC section kinds (`verse`, `preChorus`, `custom`, …).
    ///
    /// # Errors
    ///
    /// Unknown kind, or custom without a name.
    pub fn parse(
        kind: &str,
        number: Option<u16>,
        custom_name: Option<&str>,
    ) -> Result<Self, String> {
        match kind.trim() {
            "intro" => Ok(Self::Intro),
            "verse" => Ok(Self::Verse { number }),
            "preChorus" | "pre-chorus" | "prechorus" => Ok(Self::PreChorus),
            "chorus" => Ok(Self::Chorus { number }),
            "bridge" => Ok(Self::Bridge),
            "instrumental" => Ok(Self::Instrumental),
            "solo" => Ok(Self::Solo),
            "outro" => Ok(Self::Outro),
            "custom" => {
                let name = custom_name
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "Custom section needs a name.".to_string())?;
                Ok(Self::Custom {
                    name: name.to_string(),
                })
            }
            other => Err(format!("Unknown section kind '{other}'.")),
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
        assert_eq!(
            SectionLabel::parse("verse", Some(2), None)
                .unwrap()
                .display_name(),
            "Verse 2"
        );
        assert_eq!(
            SectionLabel::parse("custom", None, Some("Breakdown"))
                .unwrap()
                .display_name(),
            "Breakdown"
        );
        assert!(SectionLabel::parse("custom", None, None).is_err());
    }
}
