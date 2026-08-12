use std::io::{Cursor, Write};

use tonic_domain::{MeasureEvent, SourceFormat};
use tonic_import::{
    import, import_auto, import_bytes, import_musicxml, import_musicxml_bytes, ImportFormat,
    WarningKind, UNSUPPORTED_MUSICXML_MESSAGE,
};

const TWINKLE: &str = include_str!("../fixtures/musicxml/twinkle.musicxml");
const UNSUPPORTED: &str = include_str!("../fixtures/musicxml/unsupported.musicxml");

#[test]
fn imports_partwise_score_metadata_and_notes() {
    let result = import_musicxml(TWINKLE, "twinkle");
    assert!(!result.has_issues(), "{:?}", result.warnings);
    let song = &result.song;

    assert_eq!(song.title(), "Twinkle");
    assert_eq!(song.artist(), Some("Traditional"));
    assert_eq!(song.original_key().unwrap().symbol(), "C");
    assert_eq!(song.performance_key().unwrap().symbol(), "C");
    assert_eq!(song.time_signature().unwrap().symbol(), "4/4");
    assert_eq!(song.tempo().unwrap().bpm(), 90);
    assert_eq!(song.source().format(), &SourceFormat::MusicXml);
    assert_eq!(song.source().original_content(), Some(TWINKLE));

    let score = song.score().expect("score");
    assert_eq!(score.parts.len(), 1);
    assert_eq!(score.parts[0].name, "Voice");
    let measure = &score.parts[0].measures[0];
    assert_eq!(measure.number, 1);
    let notes: Vec<_> = measure
        .events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(note) => Some(note.pitch.note.symbol()),
            _ => None,
        })
        .collect();
    assert_eq!(notes, ["C", "C", "G"]);
    assert!(measure
        .events
        .iter()
        .any(|event| matches!(event, MeasureEvent::Rest(_))));

    let line = &song.sections()[0].lines()[0];
    assert_eq!(line.lyric_text(), "Twin kle star");
    let chords: Vec<_> = line
        .chord_tokens()
        .map(|token| token.chord().symbol())
        .collect();
    assert_eq!(chords, ["C", "G"]);
}

#[test]
fn auto_detects_musicxml_and_preserves_usable_unsupported_notes() {
    assert_eq!(
        import_auto(TWINKLE, "auto").song.title(),
        import(TWINKLE, ImportFormat::MusicXml, "auto").song.title()
    );

    let result = import_musicxml(UNSUPPORTED, "orn");
    assert!(result.has_issues());
    assert_eq!(result.summary_message(), Some(UNSUPPORTED_MUSICXML_MESSAGE));
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.kind == WarningKind::UnsupportedFeature));
    let score = result.song.score().expect("score");
    let notes: Vec<_> = score.parts[0].measures[0]
        .events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(note) => Some(note.pitch.note.symbol()),
            _ => None,
        })
        .collect();
    assert_eq!(notes, ["C", "D", "E"]);
}

#[test]
fn malformed_musicxml_keeps_a_usable_song() {
    let result = import_musicxml("<not-a-score>", "bad");
    assert!(result.has_issues());
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.kind == WarningKind::MalformedInput));
    assert_eq!(result.song.title(), "Untitled score");
    assert!(result.song.score().is_none());
}

#[test]
fn imports_mxl_archive() {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<container>
  <rootfiles>
    <rootfile full-path="score.xml" media-type="application/vnd.recordare.musicxml+xml"/>
  </rootfiles>
</container>"#,
    )
    .unwrap();
    zip.start_file("score.xml", options).unwrap();
    zip.write_all(TWINKLE.as_bytes()).unwrap();
    let bytes = zip.finish().unwrap().into_inner();

    let result = import_musicxml_bytes(&bytes, Some("twinkle.mxl"), "mxl-1");
    assert!(!result.has_issues(), "{:?}", result.warnings);
    assert_eq!(result.song.title(), "Twinkle");
    assert!(result.song.score().is_some());

    let via_bytes = import_bytes(&bytes, Some("twinkle.mxl"), "mxl-2");
    assert_eq!(via_bytes.song.title(), "Twinkle");
}

#[test]
fn transpose_display_does_not_rewrite_source_score() {
    let result = import_musicxml(TWINKLE, "src");
    let original = result.song.score().unwrap().clone();
    let xml_before = original.to_musicxml();
    let transposed = original.transpose_semitones(
        2,
        tonic_domain::Spelling::InKey(tonic_domain::Key::parse("D").unwrap()),
    );
    let xml_after = transposed.to_musicxml();
    assert!(xml_after.contains("<step>D</step>"), "{xml_after}");
    assert!(xml_before.contains("<step>C</step>"));
    assert!(!xml_after.contains("<step>C</step>"), "{xml_after}");
    assert_eq!(result.song.source().original_content(), Some(TWINKLE));
}
