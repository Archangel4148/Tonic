use tonic_domain::{ParseStatus, SectionLabel};
use tonic_import::{import, import_auto, ImportFormat, WarningKind};

const AMAZING: &str = include_str!("../fixtures/plaintext/amazing_grace.txt");
const MIXED: &str = include_str!("../fixtures/plaintext/mixed.txt");
const UNKNOWN: &str = include_str!("../fixtures/plaintext/unknown_chords.txt");

#[test]
fn imports_chord_over_lyrics_with_columns() {
    let result = import(AMAZING, ImportFormat::PlainText, "ag-txt");
    assert!(!result.has_issues(), "{:?}", result.warnings);
    let song = &result.song;

    assert_eq!(song.title(), "Amazing Grace");
    assert_eq!(song.artist(), Some("Traditional"));
    assert_eq!(song.original_key().unwrap().symbol(), "G");
    assert_eq!(
        song.sections()[0].label(),
        &SectionLabel::Verse { number: Some(1) }
    );

    let line = &song.sections()[0].lines()[0];
    assert_eq!(line.lyric_text(), "Amazing grace how sweet");
    let alignments = line.chord_lyric_alignments();
    assert_eq!(alignments[0].chord.symbol(), "C");
    assert_eq!(alignments[0].lyric_index, 0);
    assert_eq!(alignments[0].column, Some(0));
    assert_eq!(alignments[1].chord.symbol(), "G");
    assert!(alignments[1].column.unwrap() > 0);
    assert_eq!(alignments[1].lyric_index, alignments[1].column.unwrap());

    let line2 = &song.sections()[0].lines()[1];
    assert_eq!(line2.lyric_text(), "The sound that saved");
    assert_eq!(line2.chord_lyric_alignments()[0].chord.symbol(), "F");
    assert_eq!(line2.chord_lyric_alignments()[1].chord.symbol(), "C");
}

#[test]
fn mixed_plain_text_preserves_lyrics_without_chords() {
    let result = import(MIXED, ImportFormat::PlainText, "mixed");
    let song = &result.song;
    assert_eq!(song.title(), "Mixed Chart");
    assert_eq!(
        song.sections()[0].label(),
        &SectionLabel::Chorus { number: None }
    );

    let lyrics: Vec<_> = song.sections()[0]
        .lines()
        .iter()
        .map(tonic_domain::Line::lyric_text)
        .filter(|text| !text.is_empty())
        .collect();
    assert!(lyrics.iter().any(|text| text.contains("Hello world")));
    assert!(lyrics
        .iter()
        .any(|text| text.contains("This lyric line has no chords above it")));
    assert!(lyrics.iter().any(|text| text.contains("Standalone lyric")));
}

#[test]
fn unknown_plain_chords_are_preserved() {
    let result = import(UNKNOWN, ImportFormat::PlainText, "odd");
    assert!(result.has_issues());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.kind == WarningKind::UnrecognizedChord));

    let alignments = result.song.sections()[0].lines()[0].chord_lyric_alignments();
    assert_eq!(alignments[0].chord.symbol(), "C");
    assert_eq!(alignments[1].chord.status(), ParseStatus::Unrecognized);
    assert_eq!(alignments[1].chord.source_text(), "Xyz");
    assert_eq!(alignments[2].chord.symbol(), "G");
    assert_eq!(
        result.song.sections()[0].lines()[0].lyric_text(),
        "Hello strange world"
    );
}

#[test]
fn auto_detects_plain_text_fixture() {
    let result = import_auto(AMAZING, "auto-txt");
    assert_eq!(result.song.title(), "Amazing Grace");
}
