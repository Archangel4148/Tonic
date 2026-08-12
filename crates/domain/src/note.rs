//! Note letters, accidentals, and enharmonic spellings.

use std::fmt;

use crate::pitch::{PitchClass, Semitones};

/// Diatonic letter name.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Letter {
    #[must_use]
    pub fn from_char(ch: char) -> Option<Self> {
        match ch.to_ascii_uppercase() {
            'C' => Some(Self::C),
            'D' => Some(Self::D),
            'E' => Some(Self::E),
            'F' => Some(Self::F),
            'G' => Some(Self::G),
            'A' => Some(Self::A),
            'B' => Some(Self::B),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_char(self) -> char {
        match self {
            Self::C => 'C',
            Self::D => 'D',
            Self::E => 'E',
            Self::F => 'F',
            Self::G => 'G',
            Self::A => 'A',
            Self::B => 'B',
        }
    }

    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Self::C => 0,
            Self::D => 1,
            Self::E => 2,
            Self::F => 3,
            Self::G => 4,
            Self::A => 5,
            Self::B => 6,
        }
    }

    #[must_use]
    pub fn from_index(index: usize) -> Self {
        match index % 7 {
            0 => Self::C,
            1 => Self::D,
            2 => Self::E,
            3 => Self::F,
            4 => Self::G,
            5 => Self::A,
            _ => Self::B,
        }
    }

    /// Natural pitch class of this letter, ignoring accidentals.
    #[must_use]
    pub fn natural_pc(self) -> PitchClass {
        PitchClass::new(match self {
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
            Self::A => 9,
            Self::B => 11,
        })
    }
}

/// Accidental applied to a letter.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Accidental {
    DoubleFlat,
    Flat,
    Natural,
    Sharp,
    DoubleSharp,
}

impl Accidental {
    #[must_use]
    pub fn semitones(self) -> i32 {
        match self {
            Self::DoubleFlat => -2,
            Self::Flat => -1,
            Self::Natural => 0,
            Self::Sharp => 1,
            Self::DoubleSharp => 2,
        }
    }

    #[must_use]
    pub fn from_semitones(delta: i32) -> Option<Self> {
        match delta {
            -2 => Some(Self::DoubleFlat),
            -1 => Some(Self::Flat),
            0 => Some(Self::Natural),
            1 => Some(Self::Sharp),
            2 => Some(Self::DoubleSharp),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_sharp_family(self) -> bool {
        matches!(self, Self::Sharp | Self::DoubleSharp)
    }

    #[must_use]
    pub fn is_flat_family(self) -> bool {
        matches!(self, Self::Flat | Self::DoubleFlat)
    }

    /// ASCII accidental suffix used in canonical rendering (`#`, `b`, `##`, `bb`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DoubleFlat => "bb",
            Self::Flat => "b",
            Self::Natural => "",
            Self::Sharp => "#",
            Self::DoubleSharp => "##",
        }
    }

    /// Consumes a leading accidental from `input`. Returns `(accidental, bytes_consumed)`.
    #[must_use]
    pub fn consume(input: &str) -> (Self, usize) {
        let lower = input.as_bytes();
        if input.starts_with("##") || input.starts_with("♯♯") || input.starts_with('𝄪') {
            let len = if input.starts_with("♯♯") {
                "♯♯".len()
            } else if input.starts_with('𝄪') {
                '𝄪'.len_utf8()
            } else {
                2
            };
            return (Self::DoubleSharp, len);
        }
        if input.starts_with('#') || input.starts_with('♯') {
            return (Self::Sharp, input.chars().next().map_or(1, char::len_utf8));
        }
        if input.starts_with("bb") || input.starts_with("♭♭") || input.starts_with('𝄫') {
            let len = if input.starts_with("♭♭") {
                "♭♭".len()
            } else if input.starts_with('𝄫') {
                '𝄫'.len_utf8()
            } else {
                2
            };
            return (Self::DoubleFlat, len);
        }
        if !lower.is_empty() && (lower[0] == b'b' || input.starts_with('♭')) {
            return (Self::Flat, input.chars().next().map_or(1, char::len_utf8));
        }
        (Self::Natural, 0)
    }
}

/// A spelled note: letter plus accidental. Enharmonically equal notes compare unequal.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Note {
    letter: Letter,
    accidental: Accidental,
}

impl Note {
    #[must_use]
    pub const fn new(letter: Letter, accidental: Accidental) -> Self {
        Self { letter, accidental }
    }

    #[must_use]
    pub const fn natural(letter: Letter) -> Self {
        Self::new(letter, Accidental::Natural)
    }

    #[must_use]
    pub const fn sharp(letter: Letter) -> Self {
        Self::new(letter, Accidental::Sharp)
    }

    #[must_use]
    pub const fn flat(letter: Letter) -> Self {
        Self::new(letter, Accidental::Flat)
    }

    #[must_use]
    pub fn letter(self) -> Letter {
        self.letter
    }

    #[must_use]
    pub fn accidental(self) -> Accidental {
        self.accidental
    }

    #[must_use]
    pub fn pitch_class(self) -> PitchClass {
        PitchClass::new(i32::from(self.letter.natural_pc().value()) + self.accidental.semitones())
    }

    /// Parses a complete note token such as `C`, `F#`, `Bb`, or `E♯`.
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        let (note, rest) = Self::consume(input.trim())?;
        rest.trim().is_empty().then_some(note)
    }

    /// Parses a leading note and returns the remainder.
    #[must_use]
    pub fn consume(input: &str) -> Option<(Self, &str)> {
        let letter = Letter::from_char(input.chars().next()?)?;
        // A–G are always a single ASCII byte, even when the input letter is lowercase.
        let after_letter = &input[1..];
        let (accidental, consumed) = Accidental::consume(after_letter);
        Some((Self::new(letter, accidental), &after_letter[consumed..]))
    }

    #[must_use]
    pub fn symbol(self) -> String {
        format!("{}{}", self.letter.as_char(), self.accidental.as_str())
    }

    #[must_use]
    pub fn from_sharp_chromatic(pc: PitchClass) -> Self {
        SHARP_CHROMATIC[pc.value() as usize]
    }

    #[must_use]
    pub fn from_flat_chromatic(pc: PitchClass) -> Self {
        FLAT_CHROMATIC[pc.value() as usize]
    }

    /// Transposes the pitch class, then respells using `spelling`.
    #[must_use]
    pub fn transpose(self, semitones: Semitones, spelling: Spelling) -> Self {
        let pc = self.pitch_class().wrapping_add(semitones);
        match spelling {
            Spelling::InKey(key) => key.spell(pc),
            Spelling::PreserveAccidentalFamily => {
                if self.accidental.is_flat_family() {
                    Self::from_flat_chromatic(pc)
                } else {
                    Self::from_sharp_chromatic(pc)
                }
            }
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.symbol())
    }
}

/// How a transposed pitch class should be spelled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Spelling {
    /// Prefer the destination key's diatonic spelling, then that key's accidental family.
    InKey(crate::key::Key),
    /// Keep flats as flats and sharps/naturals in the sharp chromatic family.
    PreserveAccidentalFamily,
}

const SHARP_CHROMATIC: [Note; 12] = [
    Note::natural(Letter::C),
    Note::sharp(Letter::C),
    Note::natural(Letter::D),
    Note::sharp(Letter::D),
    Note::natural(Letter::E),
    Note::natural(Letter::F),
    Note::sharp(Letter::F),
    Note::natural(Letter::G),
    Note::sharp(Letter::G),
    Note::natural(Letter::A),
    Note::sharp(Letter::A),
    Note::natural(Letter::B),
];

const FLAT_CHROMATIC: [Note; 12] = [
    Note::natural(Letter::C),
    Note::flat(Letter::D),
    Note::natural(Letter::D),
    Note::flat(Letter::E),
    Note::natural(Letter::E),
    Note::natural(Letter::F),
    Note::flat(Letter::G),
    Note::natural(Letter::G),
    Note::flat(Letter::A),
    Note::natural(Letter::A),
    Note::flat(Letter::B),
    Note::natural(Letter::B),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_spellings() {
        let cases = [
            ("C", Letter::C, Accidental::Natural, PitchClass::C),
            ("c#", Letter::C, Accidental::Sharp, PitchClass::CS),
            ("Db", Letter::D, Accidental::Flat, PitchClass::CS),
            ("D#", Letter::D, Accidental::Sharp, PitchClass::DS),
            ("Eb", Letter::E, Accidental::Flat, PitchClass::DS),
            ("E#", Letter::E, Accidental::Sharp, PitchClass::F),
            ("Fb", Letter::F, Accidental::Flat, PitchClass::E),
            ("F#", Letter::F, Accidental::Sharp, PitchClass::FS),
            ("Gb", Letter::G, Accidental::Flat, PitchClass::FS),
            ("G#", Letter::G, Accidental::Sharp, PitchClass::GS),
            ("Ab", Letter::A, Accidental::Flat, PitchClass::GS),
            ("A#", Letter::A, Accidental::Sharp, PitchClass::AS),
            ("Bb", Letter::B, Accidental::Flat, PitchClass::AS),
            ("B#", Letter::B, Accidental::Sharp, PitchClass::C),
            ("Cb", Letter::C, Accidental::Flat, PitchClass::B),
            ("F##", Letter::F, Accidental::DoubleSharp, PitchClass::G),
            ("Gbb", Letter::G, Accidental::DoubleFlat, PitchClass::F),
            ("C♯", Letter::C, Accidental::Sharp, PitchClass::CS),
            ("D♭", Letter::D, Accidental::Flat, PitchClass::CS),
        ];

        for (input, letter, accidental, pc) in cases {
            let note = Note::parse(input).unwrap_or_else(|| panic!("expected to parse {input}"));
            assert_eq!(note.letter(), letter, "{input}");
            assert_eq!(note.accidental(), accidental, "{input}");
            assert_eq!(note.pitch_class(), pc, "{input}");
        }
    }

    #[test]
    fn enharmonic_notes_share_pitch_class_but_not_equality() {
        let fs = Note::parse("F#").unwrap();
        let gb = Note::parse("Gb").unwrap();
        assert_eq!(fs.pitch_class(), gb.pitch_class());
        assert_ne!(fs, gb);
        assert_eq!(fs.symbol(), "F#");
        assert_eq!(gb.symbol(), "Gb");
    }

    #[test]
    fn rejects_invalid_notes() {
        assert!(Note::parse("").is_none());
        assert!(Note::parse("H").is_none());
        assert!(Note::parse("1").is_none());
        assert!(Note::parse("Foo").is_none());
        assert!(Note::parse("C#extra").is_none());
    }

    #[test]
    fn preserve_family_transpose() {
        let fs = Note::parse("F#").unwrap();
        let bb = Note::parse("Bb").unwrap();
        assert_eq!(
            fs.transpose(Semitones::new(2), Spelling::PreserveAccidentalFamily)
                .symbol(),
            "G#"
        );
        assert_eq!(
            bb.transpose(Semitones::new(2), Spelling::PreserveAccidentalFamily)
                .symbol(),
            "C"
        );
        assert_eq!(
            Note::natural(Letter::B)
                .transpose(Semitones::new(2), Spelling::PreserveAccidentalFamily)
                .symbol(),
            "C#"
        );
    }
}
