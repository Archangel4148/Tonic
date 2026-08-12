//! Structured chord model. The symbol is never stored as the only representation.

use std::fmt;

use crate::note::Note;

/// How completely a chord symbol was recognized.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ParseStatus {
    FullyRecognized,
    PartiallyRecognized,
    Unrecognized,
}

/// Triad quality. Suspensions are stored separately and replace the third when present.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum Quality {
    #[default]
    Major,
    Minor,
    Diminished,
    Augmented,
}

/// Seventh quality. Dominant means a minor seventh (`b7`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Seventh {
    Dominant,
    Major,
    Diminished,
}

/// Upper extension implied or written in the symbol.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Extension {
    Sixth,
    Ninth,
    Eleventh,
    Thirteenth,
}

/// Alteration of a chord tone, e.g. `b5` or `#11`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Alteration {
    degree: u8,
    semitone_delta: i8,
}

impl Alteration {
    #[must_use]
    pub fn new(degree: u8, semitone_delta: i8) -> Self {
        Self {
            degree,
            semitone_delta,
        }
    }

    #[must_use]
    pub fn flat(degree: u8) -> Self {
        Self::new(degree, -1)
    }

    #[must_use]
    pub fn sharp(degree: u8) -> Self {
        Self::new(degree, 1)
    }

    #[must_use]
    pub fn degree(self) -> u8 {
        self.degree
    }

    #[must_use]
    pub fn semitone_delta(self) -> i8 {
        self.semitone_delta
    }

    #[must_use]
    pub fn symbol(self) -> String {
        let accidental = match self.semitone_delta {
            -2 => "bb",
            -1 => "b",
            1 => "#",
            2 => "##",
            0 => "",
            delta if delta < 0 => "b",
            _ => "#",
        };
        format!("{accidental}{}", self.degree)
    }
}

/// Suspended second or fourth, replacing the triad third.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Suspension {
    Sus2,
    Sus4,
}

impl Suspension {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sus2 => "sus2",
            Self::Sus4 => "sus4",
        }
    }
}

/// Explicit added tone that does not imply a seventh (`add9`, `add4`, …).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AddedTone {
    Add2,
    Add4,
    Add6,
    Add9,
    Add11,
    Add13,
}

impl AddedTone {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Add2 => "add2",
            Self::Add4 => "add4",
            Self::Add6 => "add6",
            Self::Add9 => "add9",
            Self::Add11 => "add11",
            Self::Add13 => "add13",
        }
    }
}

/// A parsed chord with independent musical components.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chord {
    root: Option<Note>,
    quality: Quality,
    seventh: Option<Seventh>,
    extensions: Vec<Extension>,
    alterations: Vec<Alteration>,
    suspension: Option<Suspension>,
    added: Vec<AddedTone>,
    bass: Option<Note>,
    source_text: String,
    status: ParseStatus,
    unparsed_tail: String,
}

impl Chord {
    #[must_use]
    pub fn unrecognized(source_text: impl Into<String>) -> Self {
        let source_text = source_text.into();
        Self {
            root: None,
            quality: Quality::Major,
            seventh: None,
            extensions: Vec::new(),
            alterations: Vec::new(),
            suspension: None,
            added: Vec::new(),
            bass: None,
            unparsed_tail: source_text.clone(),
            source_text,
            status: ParseStatus::Unrecognized,
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        root: Option<Note>,
        quality: Quality,
        seventh: Option<Seventh>,
        extensions: Vec<Extension>,
        alterations: Vec<Alteration>,
        suspension: Option<Suspension>,
        added: Vec<AddedTone>,
        bass: Option<Note>,
        source_text: impl Into<String>,
        unparsed_tail: impl Into<String>,
        status: ParseStatus,
    ) -> Self {
        Self {
            root,
            quality,
            seventh,
            extensions,
            alterations,
            suspension,
            added,
            bass,
            source_text: source_text.into(),
            status,
            unparsed_tail: unparsed_tail.into(),
        }
    }

    #[must_use]
    pub fn root(&self) -> Option<Note> {
        self.root
    }

    #[must_use]
    pub fn quality(&self) -> Quality {
        self.quality
    }

    #[must_use]
    pub fn seventh(&self) -> Option<Seventh> {
        self.seventh
    }

    #[must_use]
    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }

    #[must_use]
    pub fn alterations(&self) -> &[Alteration] {
        &self.alterations
    }

    #[must_use]
    pub fn suspension(&self) -> Option<Suspension> {
        self.suspension
    }

    #[must_use]
    pub fn added(&self) -> &[AddedTone] {
        &self.added
    }

    #[must_use]
    pub fn bass(&self) -> Option<Note> {
        self.bass
    }

    #[must_use]
    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    #[must_use]
    pub fn status(&self) -> ParseStatus {
        self.status
    }

    #[must_use]
    pub fn unparsed_tail(&self) -> &str {
        &self.unparsed_tail
    }

    pub(crate) fn set_root(&mut self, root: Option<Note>) {
        self.root = root;
    }

    pub(crate) fn set_bass(&mut self, bass: Option<Note>) {
        self.bass = bass;
    }

    pub(crate) fn set_source_text(&mut self, source_text: String) {
        self.source_text = source_text;
    }

    /// Canonical ASCII symbol for the recognized components.
    ///
    /// Unrecognized chords return the original source text so content is not lost.
    #[must_use]
    pub fn symbol(&self) -> String {
        let Some(root) = self.root else {
            return self.source_text.clone();
        };

        let mut out = root.symbol();
        match self.quality {
            Quality::Major => {}
            Quality::Minor => out.push('m'),
            Quality::Diminished => out.push_str("dim"),
            Quality::Augmented => out.push_str("aug"),
        }

        let has_sixth = self.extensions.contains(&Extension::Sixth);
        let has_ninth = self.extensions.contains(&Extension::Ninth);
        let has_eleventh = self.extensions.contains(&Extension::Eleventh);
        let has_thirteenth = self.extensions.contains(&Extension::Thirteenth);

        match self.seventh {
            Some(Seventh::Major) if has_thirteenth => out.push_str("maj13"),
            Some(Seventh::Major) if has_eleventh => out.push_str("maj11"),
            Some(Seventh::Major) if has_ninth => out.push_str("maj9"),
            Some(Seventh::Major) => out.push_str("maj7"),
            Some(Seventh::Diminished) => out.push('7'),
            Some(Seventh::Dominant) if has_thirteenth => out.push_str("13"),
            Some(Seventh::Dominant) if has_eleventh => out.push_str("11"),
            Some(Seventh::Dominant) if has_ninth => out.push('9'),
            Some(Seventh::Dominant) => out.push('7'),
            None if has_sixth && has_ninth => out.push_str("69"),
            None if has_sixth => out.push('6'),
            None if has_thirteenth => out.push_str("13"),
            None if has_eleventh => out.push_str("11"),
            None if has_ninth => out.push('9'),
            None => {}
        }

        if let Some(sus) = self.suspension {
            out.push_str(sus.as_str());
        }

        for alteration in &self.alterations {
            out.push_str(&alteration.symbol());
        }
        for added in &self.added {
            out.push_str(added.as_str());
        }
        if let Some(bass) = self.bass {
            out.push('/');
            out.push_str(&bass.symbol());
        }
        out
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.symbol())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::{Letter, Note};

    #[test]
    fn unrecognized_preserves_source_text() {
        let chord = Chord::unrecognized("N.C.");
        assert_eq!(chord.status(), ParseStatus::Unrecognized);
        assert_eq!(chord.symbol(), "N.C.");
        assert!(chord.root().is_none());
    }

    #[test]
    fn formats_minor_seven_flat_five_slash() {
        let chord = Chord::new(
            Some(Note::sharp(Letter::F)),
            Quality::Minor,
            Some(Seventh::Dominant),
            Vec::new(),
            vec![Alteration::flat(5)],
            None,
            Vec::new(),
            Some(Note::sharp(Letter::C)),
            "F#m7b5/C#",
            "",
            ParseStatus::FullyRecognized,
        );
        assert_eq!(chord.symbol(), "F#m7b5/C#");
    }
}
