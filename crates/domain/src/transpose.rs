//! Chord transposition. Only pitch-bearing components change.

use crate::chord::{Chord, ParseStatus};
use crate::key::Key;
use crate::note::Spelling;
use crate::pitch::Semitones;

/// Transpose `chord` by a signed semitone offset.
///
/// Accidentals keep their family: `F#` stays sharp-side, `Bb` stays flat-side.
#[must_use]
pub fn transpose_semitones(chord: &Chord, semitones: i32) -> Chord {
    transpose_with(
        chord,
        Semitones::new(semitones),
        Spelling::PreserveAccidentalFamily,
    )
}

/// Transpose from `source_key` to `target_key`, spelling pitches in the target key.
#[must_use]
pub fn transpose_to_key(chord: &Chord, source_key: Key, target_key: Key) -> Chord {
    let semitones = source_key.semitones_to(target_key);
    transpose_with(chord, semitones, Spelling::InKey(target_key))
}

fn transpose_with(chord: &Chord, semitones: Semitones, spelling: Spelling) -> Chord {
    if chord.status() == ParseStatus::Unrecognized || chord.root().is_none() {
        return chord.clone();
    }

    let mut out = chord.clone();
    out.set_root(chord.root().map(|note| note.transpose(semitones, spelling)));
    out.set_bass(chord.bass().map(|note| note.transpose(semitones, spelling)));
    out.set_source_text(out.symbol());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_chord;

    fn symbol_after(input: &str, semitones: i32) -> String {
        transpose_semitones(&parse_chord(input), semitones).symbol()
    }

    #[test]
    fn acceptance_semitone_examples() {
        assert_eq!(symbol_after("C", 2), "D");
        assert_eq!(symbol_after("Cm", 2), "Dm");
        assert_eq!(symbol_after("F#", 2), "G#");
        assert_eq!(symbol_after("Bb", 2), "C");
        assert_eq!(symbol_after("G/B", 2), "A/C#");
        assert_eq!(symbol_after("F#m7b5/C#", 2), "G#m7b5/D#");
    }

    #[test]
    fn negative_and_wrapping_offsets() {
        assert_eq!(symbol_after("D", -2), "C");
        assert_eq!(symbol_after("C", 14), "D");
        assert_eq!(symbol_after("C", -1), "B");
        assert_eq!(symbol_after("C7b9", 2), "D7b9");
        assert_eq!(symbol_after("Cmaj7#11", -1), "Bmaj7#11");
    }

    #[test]
    fn only_pitch_bearing_parts_change() {
        let transposed = transpose_semitones(&parse_chord("Cm7b5/G"), 3);
        assert_eq!(transposed.symbol(), "D#m7b5/A#");
        assert_eq!(transposed.quality(), parse_chord("Cm7b5/G").quality());
        assert_eq!(transposed.seventh(), parse_chord("Cm7b5/G").seventh());
        assert_eq!(
            transposed.alterations(),
            parse_chord("Cm7b5/G").alterations()
        );
    }

    #[test]
    fn unrecognized_chords_are_unchanged() {
        let original = parse_chord("N.C.");
        let transposed = transpose_semitones(&original, 5);
        assert_eq!(transposed, original);
        assert_eq!(transposed.symbol(), "N.C.");
    }

    #[test]
    fn key_to_key_uses_destination_spelling() {
        let source = Key::parse("C").unwrap();
        let target = Key::parse("Ab").unwrap();
        assert_eq!(
            transpose_to_key(&parse_chord("C"), source, target).symbol(),
            "Ab"
        );
        assert_eq!(
            transpose_to_key(&parse_chord("G"), source, target).symbol(),
            "Eb"
        );
        assert_eq!(
            transpose_to_key(&parse_chord("G#"), source, target).symbol(),
            "E"
        );
        assert_eq!(
            transpose_to_key(&parse_chord("F#m"), source, target).symbol(),
            "Dm"
        );
        assert_eq!(
            transpose_to_key(&parse_chord("D/F#"), source, target).symbol(),
            "Bb/D"
        );
    }

    #[test]
    fn a_major_to_f_major_spells_with_flats() {
        let source = Key::parse("A").unwrap();
        let target = Key::parse("F").unwrap();
        assert_eq!(
            transpose_to_key(&parse_chord("F#m"), source, target).symbol(),
            "Dm"
        );
        assert_eq!(
            transpose_to_key(&parse_chord("C#"), source, target).symbol(),
            "A"
        );
    }
}
