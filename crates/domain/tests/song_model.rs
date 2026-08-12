//! Phase 3: canonical song model without depending on raw source text.

use tonic_domain::{
    parse_chord, ChordToken, Key, Line, LineToken, LyricToken, Section, SectionLabel, Song, SongId,
    SongSource, Tempo, TimeSignature, Timestamp,
};

fn amazing_grace() -> Song {
    let source = "{title: Amazing Grace}\n{key: G}\n\n[G]Amazing grace, how [D]sweet the sound\nThat [Em]saved a wretch like [C]me\n";

    let verse = Section::new(
        SectionLabel::Verse { number: Some(1) },
        vec![
            Line::new(vec![
                LineToken::Chord(ChordToken::new(parse_chord("G"))),
                LineToken::Lyric(LyricToken::new("Amazing grace, how ")),
                LineToken::Chord(ChordToken::new(parse_chord("D"))),
                LineToken::Lyric(LyricToken::new("sweet the sound")),
            ]),
            Line::new(vec![
                LineToken::Lyric(LyricToken::new("That ")),
                LineToken::Chord(ChordToken::new(parse_chord("Em"))),
                LineToken::Lyric(LyricToken::new("saved a wretch like ")),
                LineToken::Chord(ChordToken::new(parse_chord("C"))),
                LineToken::Lyric(LyricToken::new("me")),
            ]),
        ],
    );

    Song::builder(SongId::new("amazing-grace"), "Amazing Grace")
        .artist("Traditional")
        .original_key(Key::parse("G").unwrap())
        .performance_key(Key::parse("G").unwrap())
        .tempo(Tempo::new(72).unwrap())
        .time_signature(TimeSignature::new(3, 4).unwrap())
        .source(SongSource::chordpro(source))
        .notes("Hymn")
        .created_at(Timestamp::from_secs(1_700_000_000))
        .updated_at(Timestamp::from_secs(1_700_000_100))
        .section(verse)
        .build()
}

#[test]
fn song_is_usable_without_raw_source_text() {
    let verse = Section::new(
        SectionLabel::Custom {
            name: "Breakdown".into(),
        },
        vec![Line::chord_over_lyrics(
            "Amazing grace how sweet",
            [(parse_chord("C"), 0), (parse_chord("G"), 11)],
        )],
    );

    let song = Song::builder("manual-1", "Untitled").section(verse).build();

    assert!(song.source().original_content().is_none());
    assert_eq!(song.sections()[0].label().display_name(), "Breakdown");
    assert_eq!(
        song.sections()[0].lines()[0].lyric_text(),
        "Amazing grace how sweet"
    );
    let alignments = song.sections()[0].lines()[0].chord_lyric_alignments();
    assert_eq!(alignments[0].chord.symbol(), "C");
    assert_eq!(alignments[0].lyric_index, 0);
    assert_eq!(alignments[1].chord.symbol(), "G");
    assert_eq!(alignments[1].lyric_index, 11);
    assert_eq!(alignments[1].column, Some(11));
}

#[test]
fn chord_and_lyric_positions_survive_json_round_trip() {
    let song = amazing_grace();
    let json = song.to_json().expect("serialize");
    let restored = Song::from_json(&json).expect("deserialize");
    assert_eq!(restored, song);

    let line = &restored.sections()[0].lines()[0];
    assert_eq!(line.lyric_text(), "Amazing grace, how sweet the sound");
    let alignments = line.chord_lyric_alignments();
    assert_eq!(alignments[0].chord.symbol(), "G");
    assert_eq!(alignments[0].lyric_index, 0);
    assert_eq!(alignments[1].chord.symbol(), "D");
    assert_eq!(
        alignments[1].lyric_index,
        "Amazing grace, how ".chars().count() as u32
    );
    assert!(json.contains("\"quality\":\"major\"") || json.contains("\"quality\": \"major\""));
}

#[test]
fn changing_performance_key_does_not_rewrite_written_chords_or_source() {
    let mut song = amazing_grace();
    let original_source = song.source().original_content().unwrap().to_string();
    song.set_performance_key(Some(Key::parse("Bb").unwrap()));

    let written = song.sections()[0].lines()[0]
        .chord_tokens()
        .next()
        .unwrap()
        .chord();
    assert_eq!(written.symbol(), "G");
    assert_eq!(song.display_chord(written).symbol(), "Bb");
    assert_eq!(song.display_chord(&parse_chord("D")).symbol(), "F");
    assert_eq!(song.display_chord(&parse_chord("Em")).symbol(), "Gm");
    assert_eq!(song.display_chord(&parse_chord("C")).symbol(), "Eb");
    assert_eq!(
        song.source().original_content(),
        Some(original_source.as_str())
    );
    assert_eq!(song.original_key().unwrap().symbol(), "G");
    assert_eq!(song.performance_key().unwrap().symbol(), "Bb");
}
