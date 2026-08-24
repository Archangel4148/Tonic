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
fn bar_heavy_progressions_are_chord_lines_not_lyrics() {
    // Bars can outnumber chord tokens; this must still be chords (so they transpose).
    let input = "[Intro]\n| Am | C | D | F |\n| Am | E | Am | E |\n";
    let result = import(input, ImportFormat::PlainText, "bars-heavy");
    let intro = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Intro))
        .expect("intro");
    assert_eq!(intro.lines().len(), 2);
    for line in intro.lines() {
        assert!(
            line.lyric_text().trim().is_empty(),
            "expected chord-only line, got lyrics {:?}",
            line.lyric_text()
        );
        assert!(
            line.chord_tokens().count() >= 4,
            "expected chords, got {}",
            line.chord_tokens().count()
        );
        assert!(line
            .chord_tokens()
            .all(|token| token.chord().status() == ParseStatus::FullyRecognized));
    }
}

#[test]
fn bar_lines_in_chord_progressions_are_not_warnings() {
    let input = "title: Rising\n\n[Intro]\n| Am | C | D | F |\n| Am | E | Am | E |\n";
    let result = import(input, ImportFormat::PlainText, "bars");
    assert!(
        !result
            .warnings
            .iter()
            .any(|warning| warning.kind == WarningKind::UnrecognizedChord),
        "{:?}",
        result.warnings
    );
    let intro = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Intro))
        .expect("intro");
    let symbols: Vec<_> = intro.lines()[0]
        .chord_tokens()
        .map(|token| token.chord().symbol())
        .collect();
    assert_eq!(symbols, vec!["Am", "C", "D", "F"]);
}

#[test]
fn parenthesized_chord_groups_are_recognized_and_keep_parens() {
    let input = "[Intro]\n( Am Dm G ) C\n";
    let result = import(input, ImportFormat::PlainText, "paren-groups");
    let intro = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Intro))
        .expect("intro");
    let symbols: Vec<_> = intro.lines()[0]
        .chord_tokens()
        .map(|token| token.chord().symbol())
        .collect();
    assert_eq!(symbols, vec!["(Am", "Dm", "G)", "C"]);
    assert!(
        intro
            .lines()[0]
            .chord_tokens()
            .all(|token| token.chord().status() == ParseStatus::FullyRecognized)
    );
    assert!(
        !result
            .warnings
            .iter()
            .any(|warning| warning.kind == WarningKind::UnrecognizedChord),
        "{:?}",
        result.warnings
    );
}

#[test]
fn auto_detects_plain_text_fixture() {
    let result = import_auto(AMAZING, "auto-txt");
    assert_eq!(result.song.title(), "Amazing Grace");
}

#[test]
fn custom_bracket_headings_become_sections() {
    let result = import(
        "[Hook]\nC     G\nwait for it\n",
        ImportFormat::PlainText,
        "hook",
    );
    assert_eq!(
        result.song.sections()[0].label(),
        &SectionLabel::Custom {
            name: "Hook".into()
        }
    );
    assert_eq!(result.song.sections()[0].lines()[0].lyric_text(), "wait for it");
}

#[test]
fn export_round_trips_chord_columns() {
    let result = import(AMAZING, ImportFormat::PlainText, "ag-txt");
    let chart = tonic_import::export_plain_text(&result.song);
    assert!(chart.contains("[Verse 1]"), "{chart}");
    assert!(chart.contains("Amazing grace how sweet"), "{chart}");
    let again = import(&chart, ImportFormat::PlainText, "ag-round");
    let first = &result.song.sections()[0].lines()[0];
    let second = &again.song.sections()[0].lines()[0];
    assert_eq!(first.lyric_text(), second.lyric_text());
    let a = first.chord_lyric_alignments();
    let b = second.chord_lyric_alignments();
    assert_eq!(a.len(), b.len());
    assert_eq!(a[0].chord.symbol(), b[0].chord.symbol());
    assert_eq!(a[1].chord.symbol(), b[1].chord.symbol());
    assert_eq!(a[0].column, b[0].column);
    assert_eq!(a[1].column, b[1].column);
}
