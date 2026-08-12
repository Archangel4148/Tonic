//! Major/minor keys and key-aware enharmonic spelling.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::note::{Accidental, Letter, Note};
use crate::pitch::{PitchClass, Semitones};

/// Scale mode used for diatonic spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Mode {
    Major,
    Minor,
}

/// A tonal center with a spelled tonic and mode.
///
/// `C# major` and `Db major` share a pitch class but are different keys because
/// they imply different accidental families.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Key {
    tonic: Note,
    mode: Mode,
}

impl Key {
    #[must_use]
    pub const fn major(tonic: Note) -> Self {
        Self {
            tonic,
            mode: Mode::Major,
        }
    }

    #[must_use]
    pub const fn minor(tonic: Note) -> Self {
        Self {
            tonic,
            mode: Mode::Minor,
        }
    }

    #[must_use]
    pub fn tonic(self) -> Note {
        self.tonic
    }

    #[must_use]
    pub fn mode(self) -> Mode {
        self.mode
    }

    #[must_use]
    pub fn pitch_class(self) -> PitchClass {
        self.tonic.pitch_class()
    }

    /// Parses compact musician names such as `C`, `Am`, `F# minor`, or `Bb major`.
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        let (tonic, rest) = Note::consume(trimmed)?;
        let rest = rest.trim();
        if rest.is_empty() {
            return Some(Self::major(tonic));
        }
        let lower = rest.to_ascii_lowercase();
        match lower.as_str() {
            "m" | "min" | "minor" => Some(Self::minor(tonic)),
            "maj" | "major" => Some(Self::major(tonic)),
            _ => None,
        }
    }

    #[must_use]
    pub fn symbol(self) -> String {
        match self.mode {
            Mode::Major => self.tonic.symbol(),
            Mode::Minor => format!("{}m", self.tonic.symbol()),
        }
    }

    /// Semitone distance from this tonic up to `other`'s tonic, in `0..12`.
    #[must_use]
    pub fn semitones_to(self, other: Self) -> Semitones {
        self.pitch_class().ascending_to(other.pitch_class())
    }

    /// Move this key by `semitones`, keeping major/minor and using common tonic spellings.
    ///
    /// `G + 1 → Ab`, `Am + 2 → Bm`. Display chords still use [`crate::transpose::transpose_to_key`].
    #[must_use]
    pub fn transpose_semitones(self, semitones: i32) -> Self {
        let pc = self.pitch_class().wrapping_add(Semitones::new(semitones));
        Self::from_pitch_class(pc, self.mode)
    }

    /// MusicXML `fifths` + major/minor. `None` when fifths is outside `-7..=7`.
    #[must_use]
    pub fn from_fifths(fifths: i32, minor: bool) -> Option<Self> {
        let index = usize::try_from(fifths + 7).ok()?;
        let symbol = if minor {
            MINOR_FIFTHS.get(index)?
        } else {
            MAJOR_FIFTHS.get(index)?
        };
        Self::parse(symbol)
    }

    /// Inverse of [`Self::from_fifths`] for common keys.
    #[must_use]
    pub fn fifths(self) -> Option<i32> {
        let symbol = self.symbol();
        let table = match self.mode {
            Mode::Minor => &MINOR_FIFTHS,
            Mode::Major => &MAJOR_FIFTHS,
        };
        table
            .iter()
            .position(|candidate| *candidate == symbol)
            .map(|index| index as i32 - 7)
    }

    /// Preferred concert-key spelling for a tonic pitch class.
    #[must_use]
    pub fn from_pitch_class(pc: PitchClass, mode: Mode) -> Self {
        let tonic = preferred_tonic(pc, mode);
        match mode {
            Mode::Major => Self::major(tonic),
            Mode::Minor => Self::minor(tonic),
        }
    }

    /// Preferred accidental family for chromatic (non-diatonic) notes.
    #[must_use]
    pub fn accidental_preference(self) -> AccidentalPreference {
        let mut sharps = 0;
        let mut flats = 0;
        for note in self.diatonic_notes() {
            if note.accidental().is_sharp_family() {
                sharps += 1;
            }
            if note.accidental().is_flat_family() {
                flats += 1;
            }
        }
        if flats > sharps {
            AccidentalPreference::Flats
        } else {
            AccidentalPreference::Sharps
        }
    }

    /// Spell `pc` using diatonic notes of this key when possible.
    ///
    /// Otherwise use the key's accidental family. This is the strategy behind
    /// "in Ab major, render Ab rather than G#".
    #[must_use]
    pub fn spell(self, pc: PitchClass) -> Note {
        for note in self.diatonic_notes() {
            if note.pitch_class() == pc {
                return note;
            }
        }
        match self.accidental_preference() {
            AccidentalPreference::Flats => Note::from_flat_chromatic(pc),
            AccidentalPreference::Sharps => Note::from_sharp_chromatic(pc),
        }
    }

    #[must_use]
    pub fn diatonic_notes(self) -> [Note; 7] {
        let intervals = match self.mode {
            Mode::Major => [0, 2, 4, 5, 7, 9, 11],
            Mode::Minor => [0, 2, 3, 5, 7, 8, 10],
        };
        let tonic_letter = self.tonic.letter().index();
        let tonic_pc = i32::from(self.tonic.pitch_class().value());
        let mut notes = [Note::natural(Letter::C); 7];
        for (i, interval) in intervals.into_iter().enumerate() {
            let letter = Letter::from_index(tonic_letter + i);
            let target_pc = (tonic_pc + interval).rem_euclid(12) as u8;
            let natural_pc = i32::from(letter.natural_pc().value());
            let delta = signed_pc_delta(natural_pc, i32::from(target_pc));
            let accidental = Accidental::from_semitones(delta).unwrap_or(Accidental::Natural);
            notes[i] = Note::new(letter, accidental);
        }
        notes
    }
}

/// Accidental family used for non-diatonic chromatic notes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccidentalPreference {
    Sharps,
    Flats,
}

fn preferred_tonic(pc: PitchClass, mode: Mode) -> Note {
    let symbols: [&str; 12] = match mode {
        Mode::Major => [
            "C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
        ],
        Mode::Minor => [
            "C", "C#", "D", "Eb", "E", "F", "F#", "G", "G#", "A", "Bb", "B",
        ],
    };
    Note::parse(symbols[pc.value() as usize]).expect("preferred tonic symbols are valid")
}

const MAJOR_FIFTHS: [&str; 15] = [
    "Cb", "Gb", "Db", "Ab", "Eb", "Bb", "F", "C", "G", "D", "A", "E", "B", "F#", "C#",
];
const MINOR_FIFTHS: [&str; 15] = [
    "Abm", "Ebm", "Bbm", "Fm", "Cm", "Gm", "Dm", "Am", "Em", "Bm", "F#m", "C#m", "G#m", "D#m",
    "A#m",
];

fn signed_pc_delta(from: i32, to: i32) -> i32 {
    let raw = (to - from).rem_euclid(12);
    if raw > 6 {
        raw - 12
    } else {
        raw
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.symbol())
    }
}

impl Serialize for Key {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.symbol())
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let symbol = String::deserialize(deserializer)?;
        Self::parse(&symbol)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid key: {symbol}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_major_and_minor_keys() {
        let cases = [
            ("C", "C", Mode::Major, PitchClass::C),
            ("Am", "Am", Mode::Minor, PitchClass::A),
            ("F#m", "F#m", Mode::Minor, PitchClass::FS),
            ("Bb major", "Bb", Mode::Major, PitchClass::AS),
            ("D minor", "Dm", Mode::Minor, PitchClass::D),
            ("Eb", "Eb", Mode::Major, PitchClass::DS),
            ("C#", "C#", Mode::Major, PitchClass::CS),
            ("Db", "Db", Mode::Major, PitchClass::CS),
        ];
        for (input, symbol, mode, pc) in cases {
            let key = Key::parse(input).unwrap_or_else(|| panic!("expected to parse {input}"));
            assert_eq!(key.mode(), mode, "{input}");
            assert_eq!(key.pitch_class(), pc, "{input}");
            assert_eq!(key.symbol(), symbol, "{input}");
        }
    }

    #[test]
    fn enharmonic_keys_are_distinct() {
        let cs = Key::parse("C#").unwrap();
        let db = Key::parse("Db").unwrap();
        assert_eq!(cs.pitch_class(), db.pitch_class());
        assert_ne!(cs, db);
        assert_eq!(cs.accidental_preference(), AccidentalPreference::Sharps);
        assert_eq!(db.accidental_preference(), AccidentalPreference::Flats);
    }

    #[test]
    fn spells_ab_not_gs_in_ab_major() {
        let ab_major = Key::parse("Ab").unwrap();
        assert_eq!(ab_major.spell(PitchClass::GS).symbol(), "Ab");
        assert_eq!(ab_major.spell(PitchClass::AS).symbol(), "Bb");
        assert_eq!(ab_major.spell(PitchClass::CS).symbol(), "Db");
    }

    #[test]
    fn spells_fs_not_gb_in_a_major() {
        let a_major = Key::parse("A").unwrap();
        assert_eq!(a_major.spell(PitchClass::FS).symbol(), "F#");
        assert_eq!(a_major.spell(PitchClass::GS).symbol(), "G#");
        assert_eq!(a_major.spell(PitchClass::CS).symbol(), "C#");
    }

    #[test]
    fn c_major_uses_natural_diatonic_spellings() {
        let c = Key::parse("C").unwrap();
        let symbols: Vec<_> = c.diatonic_notes().into_iter().map(Note::symbol).collect();
        assert_eq!(symbols, ["C", "D", "E", "F", "G", "A", "B"]);
        assert_eq!(c.spell(PitchClass::CS).symbol(), "C#");
    }

    #[test]
    fn a_minor_matches_c_major_signature() {
        let am = Key::parse("Am").unwrap();
        let symbols: Vec<_> = am.diatonic_notes().into_iter().map(Note::symbol).collect();
        assert_eq!(symbols, ["A", "B", "C", "D", "E", "F", "G"]);
        assert_eq!(am.accidental_preference(), AccidentalPreference::Sharps);
    }

    #[test]
    fn fs_major_uses_e_sharp() {
        let fs = Key::parse("F#").unwrap();
        let symbols: Vec<_> = fs.diatonic_notes().into_iter().map(Note::symbol).collect();
        assert_eq!(symbols, ["F#", "G#", "A#", "B", "C#", "D#", "E#"]);
    }

    #[test]
    fn gb_major_uses_c_flat() {
        let gb = Key::parse("Gb").unwrap();
        let symbols: Vec<_> = gb.diatonic_notes().into_iter().map(Note::symbol).collect();
        assert_eq!(symbols, ["Gb", "Ab", "Bb", "Cb", "Db", "Eb", "F"]);
    }

    #[test]
    fn rejects_invalid_keys() {
        assert!(Key::parse("").is_none());
        assert!(Key::parse("H").is_none());
        assert!(Key::parse("C dorian").is_none());
        assert!(Key::parse("major").is_none());
    }

    #[test]
    fn fifths_round_trip_common_keys() {
        assert_eq!(Key::from_fifths(0, false).unwrap().symbol(), "C");
        assert_eq!(Key::from_fifths(2, false).unwrap().symbol(), "D");
        assert_eq!(Key::from_fifths(-1, false).unwrap().symbol(), "F");
        assert_eq!(Key::from_fifths(0, true).unwrap().symbol(), "Am");
        assert_eq!(Key::parse("G").unwrap().fifths(), Some(1));
        assert_eq!(Key::parse("Bb").unwrap().fifths(), Some(-2));
        assert_eq!(Key::parse("Em").unwrap().fifths(), Some(1));
        assert!(Key::from_fifths(8, false).is_none());
    }

    #[test]
    fn transpose_semitones_uses_common_spellings() {
        assert_eq!(
            Key::parse("G").unwrap().transpose_semitones(2).symbol(),
            "A"
        );
        assert_eq!(
            Key::parse("G").unwrap().transpose_semitones(1).symbol(),
            "Ab"
        );
        assert_eq!(
            Key::parse("Am").unwrap().transpose_semitones(2).symbol(),
            "Bm"
        );
        assert_eq!(
            Key::parse("C").unwrap().transpose_semitones(-1).symbol(),
            "B"
        );
        assert_eq!(
            Key::parse("F#").unwrap().transpose_semitones(1).symbol(),
            "G"
        );
        assert_eq!(
            Key::parse("Db").unwrap().transpose_semitones(2).symbol(),
            "Eb"
        );
    }
}
