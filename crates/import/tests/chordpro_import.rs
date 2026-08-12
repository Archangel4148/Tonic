use tonic_domain::{ParseStatus, SectionLabel};
use tonic_import::{import, import_auto, ImportFormat, WarningKind, UNRECOGNIZED_CONTENT_MESSAGE};

const AMAZING: &str = include_str!("../fixtures/chordpro/amazing_grace.cho");
const UNKNOWN: &str = include_str!("../fixtures/chordpro/unknown_chords.cho");
const MALFORMED: &str = include_str!("../fixtures/chordpro/malformed.cho");

#[test]
fn imports_chordpro_metadata_sections_and_positions() {
    let result = import(AMAZING, ImportFormat::ChordPro, "ag-cho");
    assert!(!result.has_issues(), "{:?}", result.warnings);
    let song = &result.song;

    assert_eq!(song.title(), "Amazing Grace");
    assert_eq!(song.artist(), Some("Traditional"));
    assert_eq!(song.original_key().unwrap().symbol(), "G");
    assert_eq!(song.performance_key().unwrap().symbol(), "G");
    assert_eq!(song.tempo().unwrap().bpm(), 72);
    assert_eq!(song.time_signature().unwrap().symbol(), "3/4");
    assert_eq!(song.source().original_content(), Some(AMAZING));
    assert_eq!(song.sections().len(), 2);
    assert_eq!(song.sections()[0].label().display_name(), "Verse 1");
    assert_eq!(
        song.sections()[1].label(),
        &SectionLabel::Chorus { number: None }
    );

    let line = &song.sections()[0].lines()[0];
    assert_eq!(line.lyric_text(), "Amazing grace, how sweet the sound");
    let alignments = line.chord_lyric_alignments();
    assert_eq!(alignments[0].chord.symbol(), "G");
    assert_eq!(alignments[0].lyric_index, 0);
    assert_eq!(alignments[1].chord.symbol(), "D");
    assert_eq!(
        alignments[1].lyric_index,
        "Amazing grace, how ".chars().count() as u32
    );

    assert_eq!(
        song.sections()[0].lines()[1].lyric_text(),
        "That saved a wretch like me"
    );
}

#[test]
fn preserves_unknown_chords_and_slash_chords() {
    let result = import(UNKNOWN, ImportFormat::ChordPro, "jazz");
    assert!(result.has_issues());
    assert_eq!(result.summary_message(), Some(UNRECOGNIZED_CONTENT_MESSAGE));
    assert!(result
        .warnings
        .iter()
        .any(|w| w.kind == WarningKind::UnrecognizedChord));

    let line = &result.song.sections()[0].lines()[0];
    assert_eq!(line.lyric_text(), "Hello world there");
    let chords: Vec<_> = line
        .chord_tokens()
        .map(|token| (token.chord().symbol(), token.chord().status()))
        .collect();
    assert_eq!(chords[0], ("C".to_string(), ParseStatus::FullyRecognized));
    assert_eq!(chords[1].1, ParseStatus::Unrecognized);
    assert_eq!(chords[1].0, "Xyz");
    assert_eq!(chords[2], ("G".to_string(), ParseStatus::FullyRecognized));

    let slash = result.song.sections()[0].lines()[1]
        .chord_tokens()
        .next()
        .unwrap();
    assert_eq!(slash.chord().symbol(), "F#m7b5/C#");
    assert_eq!(slash.chord().status(), ParseStatus::FullyRecognized);
}

#[test]
fn malformed_chordpro_keeps_usable_content() {
    let result = import(MALFORMED, ImportFormat::ChordPro, "broken");
    assert!(result.has_issues());
    assert_eq!(result.song.title(), "Broken Song");
    assert!(!result.song.sections().is_empty());

    let lyrics: String = result
        .song
        .sections()
        .iter()
        .flat_map(|section| section.lines())
        .map(tonic_domain::Line::lyric_text)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(lyrics.contains("Hello"));
    assert!(lyrics.to_lowercase().contains("odd") || lyrics.contains("this is odd"));

    let has_c = result
        .song
        .sections()
        .iter()
        .flat_map(|section| section.lines())
        .flat_map(tonic_domain::Line::chord_tokens)
        .any(|token| token.chord().symbol() == "C");
    assert!(has_c);

    assert!(result
        .warnings
        .iter()
        .any(|w| w.kind == WarningKind::UnrecognizedDirective
            || w.kind == WarningKind::MalformedInput));
}

#[test]
fn auto_detects_chordpro_fixture() {
    let result = import_auto(AMAZING, "auto-ag");
    assert_eq!(result.song.title(), "Amazing Grace");
    assert!(!result.has_issues());
}
