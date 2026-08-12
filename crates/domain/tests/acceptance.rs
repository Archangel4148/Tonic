//! Phase 2 acceptance examples from the product spec.

use tonic_domain::{parse_chord, transpose_semitones, ParseStatus};

#[test]
fn spec_semitone_transposition_examples() {
    let cases = [
        ("C", "D"),
        ("Cm", "Dm"),
        ("F#", "G#"),
        ("Bb", "C"),
        ("G/B", "A/C#"),
        ("F#m7b5/C#", "G#m7b5/D#"),
    ];

    for (input, expected) in cases {
        let parsed = parse_chord(input);
        assert_eq!(
            parsed.status(),
            ParseStatus::FullyRecognized,
            "{input} should parse fully, tail={}",
            parsed.unparsed_tail()
        );
        assert_eq!(
            transpose_semitones(&parsed, 2).symbol(),
            expected,
            "{input} +2"
        );
    }
}

#[test]
fn valid_and_invalid_parse_cases_are_both_covered() {
    assert_eq!(
        parse_chord("Cmaj7#11").status(),
        ParseStatus::FullyRecognized
    );
    assert_eq!(parse_chord("Hello").status(), ParseStatus::Unrecognized);
    assert_eq!(
        parse_chord("C7oops").status(),
        ParseStatus::PartiallyRecognized
    );
}
