//! Ultimate Guitar web import fixtures.

use tonic_domain::{LineToken, SectionLabel, SongId};
use tonic_import::{import_web_html, recognize_web_url, WebSite};

#[test]
fn recognizes_ultimate_guitar_chord_urls() {
    assert_eq!(
        recognize_web_url(
            "https://tabs.ultimate-guitar.com/tab/traditional/amazing-grace-chords-1080922"
        ),
        Some(WebSite::UltimateGuitar)
    );
    assert_eq!(
        recognize_web_url("https://example.com/song"),
        None
    );
}

#[test]
fn parses_ultimate_guitar_fixture() {
    let html = include_str!("../fixtures/web/ultimate_guitar_amazing_grace.html");
    let result = import_web_html(
        "https://tabs.ultimate-guitar.com/tab/traditional/amazing-grace-chords-1080922",
        html,
        SongId::new("song-ug-1"),
    )
    .expect("fixture should parse");

    assert_eq!(result.song.title(), "Amazing Grace");
    assert_eq!(result.song.artist(), Some("Traditional"));
    assert_eq!(
        result.song.original_key().map(|key| key.symbol()),
        Some("G".to_string())
    );
    assert_eq!(result.song.source().website(), Some("ultimate-guitar"));
    assert_eq!(
        result.song.source().url(),
        Some("https://tabs.ultimate-guitar.com/tab/traditional/amazing-grace-chords-1080922")
    );

    let first = &result.song.sections()[0].lines()[0];
    let chords: Vec<_> = first
        .chord_tokens()
        .map(|token| token.chord().symbol())
        .collect();
    assert!(chords.iter().any(|symbol| symbol == "G"));
    assert!(chords.iter().any(|symbol| symbol == "D"));
    assert!(first.lyric_text().contains("Amazing grace"));
}

#[test]
fn parses_tab_block_chord_over_lyrics_with_column_alignment() {
    let html = include_str!("../fixtures/web/ultimate_guitar_tab_blocks.html");
    let result = import_web_html(
        "https://tabs.ultimate-guitar.com/tab/example/song-chords-627779",
        html,
        SongId::new("song-tab-blocks"),
    )
    .expect("tab-block fixture should parse");

    assert_eq!(result.song.title(), "The Gambler");
    assert_eq!(result.song.artist(), Some("Kenny Rogers"));
    assert_eq!(
        result.song.original_key().map(|key| key.symbol()),
        Some("G".to_string())
    );

    let verse = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Verse { number: Some(1) }))
        .expect("verse 1 section");
    let first_line = verse
        .lines()
        .iter()
        .find(|line| line.lyric_text().contains("summer's evenin'"))
        .expect("first lyric line");
    assert!(first_line.lyric_text().contains("summer's evenin'"));
    assert!(!first_line.lyric_text().contains("&#039;"));

    let alignments = first_line.chord_lyric_alignments();
    assert_eq!(alignments.len(), 3);
    assert_eq!(alignments[0].chord.symbol(), "G");
    assert_eq!(alignments[0].column, Some(5));
    assert_eq!(alignments[1].chord.symbol(), "C");
    assert!(alignments[1].column.unwrap() > 20);
    assert_eq!(alignments[2].chord.symbol(), "G");

    let notes = result.song.notes().unwrap_or("");
    assert!(notes.contains("Original key Eb"), "{notes}");
    assert!(
        notes.contains("The first few lines are picking"),
        "{notes}"
    );

    let annotations: Vec<_> = verse
        .lines()
        .iter()
        .flat_map(|line| {
            line.tokens().iter().filter_map(|token| match token {
                LineToken::Annotation(annotation) => Some(annotation.text().to_string()),
                _ => None,
            })
        })
        .collect();
    assert!(
        !annotations
            .iter()
            .any(|note| note.contains("Original key Eb")),
        "preamble should be song notes, not verse annotations: {annotations:?}"
    );
}

#[test]
fn parses_intro_progression_and_repeat_markers() {
    let html = include_str!("../fixtures/web/ultimate_guitar_intro_repeat.html");
    let result = import_web_html(
        "https://tabs.ultimate-guitar.com/tab/example/song-chords-46190",
        html,
        SongId::new("song-intro-repeat"),
    )
    .expect("intro/repeat fixture should parse");

    assert_eq!(result.song.title(), "Hotel California");
    assert_eq!(result.song.artist(), Some("Eagles"));

    let intro = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Intro))
        .expect("intro section");
    assert!(!intro.lines().is_empty(), "intro should have lines");
    let intro_chord_line = intro
        .lines()
        .iter()
        .find(|line| !line.chord_tokens().next().is_none())
        .expect("intro chord line");
    let intro_chords: Vec<_> = intro_chord_line
        .chord_tokens()
        .map(|token| token.chord().symbol())
        .collect();
    assert!(intro_chords.contains(&"Am".to_string()));
    assert!(intro_chords.contains(&"E7".to_string()));
    assert!(!intro_chord_line.lyric_text().contains("[ch]"));
    assert!(!intro_chord_line.lyric_text().contains("x2"));

    let intro_annotations: Vec<_> = intro
        .lines()
        .iter()
        .flat_map(|line| {
            line.tokens().iter().filter_map(|token| match token {
                LineToken::Annotation(annotation) => Some(annotation.text().to_string()),
                _ => None,
            })
        })
        .collect();
    assert!(
        !intro_annotations
            .iter()
            .any(|note| note.contains("Tabbed by") || note == "Capo 2"),
        "preamble should be song notes, not intro annotations: {intro_annotations:?}"
    );
    assert!(intro_annotations.iter().any(|note| note == "Repeat 2×"));

    let notes = result.song.notes().unwrap_or("");
    assert!(notes.contains("Tabbed by: Emrldeyzs"), "{notes}");
    assert!(notes.contains("Capo 2"), "{notes}");

    let harmonies = result
        .song
        .sections()
        .iter()
        .find(|section| {
            matches!(
                section.label(),
                SectionLabel::Custom { name } if name == "Harmonies"
            )
        })
        .expect("harmonies section");
    assert!(
        harmonies
            .lines()
            .iter()
            .any(|line| line.lyric_text().contains("fade out"))
    );
}

#[test]
fn bar_progression_lines_become_recognized_chords() {
    let html = include_str!("../fixtures/web/ultimate_guitar_bar_progressions.html");
    let result = import_web_html(
        "https://tabs.ultimate-guitar.com/tab/example/song-chords-18688",
        html,
        SongId::new("song-bars"),
    )
    .expect("bar progression fixture should parse");

    let intro = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Intro))
        .expect("intro section");
    assert!(
        !intro.lines().is_empty(),
        "intro lines: {:?}",
        intro.lines().iter().map(|line| (
            line.lyric_text(),
            line.chord_tokens().map(|t| t.chord().source_text().to_string()).collect::<Vec<_>>()
        )).collect::<Vec<_>>()
    );
    let first = &intro.lines()[0];
    assert!(
        first.lyric_text().trim().is_empty(),
        "expected chord-only line, got lyrics {:?}",
        first.lyric_text()
    );
    let chords: Vec<_> = first
        .chord_tokens()
        .map(|token| {
            (
                token.chord().symbol(),
                token.chord().status(),
                token.chord().source_text().to_string(),
            )
        })
        .collect();
    assert_eq!(
        chords
            .iter()
            .map(|(symbol, _, _)| symbol.as_str())
            .collect::<Vec<_>>(),
        vec!["Am", "C", "D", "F"]
    );
    assert!(
        chords
            .iter()
            .all(|(_, status, _)| *status == tonic_domain::ParseStatus::FullyRecognized),
        "{chords:?}"
    );

    let solo = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Solo))
        .expect("organ solo section");
    let solo_chords: Vec<_> = solo.lines()[0]
        .chord_tokens()
        .map(|token| token.chord().symbol())
        .collect();
    assert_eq!(solo_chords, vec!["Am", "C", "D", "F"]);

    let interlude = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Instrumental))
        .expect("interlude section");
    let interlude_chords: Vec<_> = interlude.lines()[0]
        .chord_tokens()
        .map(|token| (token.chord().symbol(), token.chord().status()))
        .collect();
    assert_eq!(
        interlude_chords
            .iter()
            .map(|(symbol, _)| symbol.as_str())
            .collect::<Vec<_>>(),
        vec!["(Eb)", "Eb", "C7b9", "F"]
    );
    assert!(
        interlude_chords
            .iter()
            .all(|(_, status)| *status == tonic_domain::ParseStatus::FullyRecognized),
        "{interlude_chords:?}"
    );
    assert!(
        !result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("(Eb)")
                || warning.message.contains("C7-9")),
        "unexpected warnings: {:?}",
        result.warnings
    );
}

#[test]
fn preamble_before_first_section_becomes_song_notes() {
    let html = include_str!("../fixtures/web/ultimate_guitar_preamble_notes.html");
    let result = import_web_html(
        "https://tabs.ultimate-guitar.com/tab/jeff-buckley/hallelujah-chords-328812",
        html,
        SongId::new("song-preamble"),
    )
    .expect("preamble fixture should parse");

    assert_eq!(result.song.title(), "Hallelujah");
    assert_eq!(result.song.artist(), Some("Jeff Buckley"));

    let notes = result.song.notes().unwrap_or("");
    assert!(
        notes.contains("Capo 5 for the original studio version"),
        "{notes}"
    );
    assert!(notes.contains("Capo 6 for the official video"), "{notes}");
    assert!(notes.contains("youtube.com"), "{notes}");

    assert!(
        !result
            .song
            .sections()
            .iter()
            .any(|section| matches!(section.label(), SectionLabel::Verse { number: None })),
        "unlabeled default Verse should not be created for preamble: {:?}",
        result
            .song
            .sections()
            .iter()
            .map(|section| format!("{:?}", section.label()))
            .collect::<Vec<_>>()
    );

    let verse1 = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Verse { number: Some(1) }))
        .expect("verse 1");
    assert!(verse1
        .lines()
        .iter()
        .any(|line| line.lyric_text().contains("secret chord")));
    assert!(
        !verse1
            .lines()
            .iter()
            .any(|line| line.lyric_text().contains("Capo")),
        "capo preamble should not appear as verse lyrics"
    );
}

#[test]
fn unlabeled_ug_chart_is_not_swallowed_as_notes() {
    let html = include_str!("../fixtures/web/ultimate_guitar_unlabeled_chart.html");
    let result = import_web_html(
        "https://tabs.ultimate-guitar.com/tab/frank-sinatra/fly-me-to-the-moon-chords-335196",
        html,
        SongId::new("song-unlabeled"),
    )
    .expect("unlabeled fixture should parse");

    assert_eq!(result.song.title(), "Fly Me To The Moon");
    assert_eq!(result.song.artist(), Some("Frank Sinatra"));
    let notes = result.song.notes().unwrap_or("");
    assert!(notes.contains("Fly Me To The Moon chords"), "{notes}");
    assert!(notes.contains("wikipedia.org"), "{notes}");

    let has_lyric = result.song.sections().iter().any(|section| {
        section
            .lines()
            .iter()
            .any(|line| line.lyric_text().contains("Fly me to the moon"))
    });
    assert!(has_lyric, "expected lyric lines, notes={notes:?}");

    let intro = result
        .song
        .sections()
        .iter()
        .find(|section| matches!(section.label(), SectionLabel::Intro))
        .expect("intro from parenthesized group");
    let symbols: Vec<_> = intro.lines()[0]
        .chord_tokens()
        .map(|token| token.chord().symbol())
        .collect();
    assert_eq!(symbols, vec!["(Am", "Dm", "G)", "C"]);
}

#[test]
fn rejects_pages_without_js_store() {
    let err = import_web_html(
        "https://tabs.ultimate-guitar.com/tab/x/y-chords-1",
        "<html><body>No data</body></html>",
        SongId::new("song-ug-2"),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Could not find song data"));
}

#[test]
fn ellipsis_in_lyrics_does_not_crash_import() {
    let html = include_str!("../fixtures/web/ultimate_guitar_ellipsis.html");
    let result = import_web_html(
        "https://tabs.ultimate-guitar.com/tab/three-dog-night/joy-to-the-world-chords-692363",
        html,
        SongId::new("song-ellipsis"),
    )
    .expect("ellipsis fixture should parse");
    assert_eq!(result.song.title(), "Joy To The World");
    assert!(
        result
            .song
            .sections()
            .iter()
            .any(|section| section
                .lines()
                .iter()
                .any(|line| line.lyric_text().contains("Joy to the world"))),
        "expected chorus lyrics"
    );
}
