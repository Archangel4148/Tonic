//! Extensible chord-symbol parser. Not a fixed dictionary of whole symbols.

use crate::chord::{
    AddedTone, Alteration, Chord, Extension, ParseStatus, Quality, Seventh, Suspension,
};
use crate::note::Note;

/// Parse `input` into a structured [`Chord`].
///
/// Never fails destructively: unrecognized text is preserved on the returned chord.
#[must_use]
pub fn parse_chord(input: &str) -> Chord {
    let original = input.to_string();
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Chord::unrecognized(original);
    }

    let mut cursor = Cursor::new(trimmed);
    let Some(root) = cursor.consume_note() else {
        return Chord::unrecognized(original);
    };

    let mut quality = Quality::Major;
    let mut seventh = None;
    let mut extensions = Vec::new();
    let mut alterations = Vec::new();
    let mut suspension = None;
    let mut added = Vec::new();

    parse_quality(
        &mut cursor,
        &mut quality,
        &mut seventh,
        &mut extensions,
        &mut alterations,
    );
    parse_suspension(&mut cursor, &mut suspension);
    parse_extensions(&mut cursor, quality, &mut seventh, &mut extensions);
    parse_suspension(&mut cursor, &mut suspension);
    parse_extensions(&mut cursor, quality, &mut seventh, &mut extensions);
    parse_alterations_and_adds(&mut cursor, &mut alterations, &mut added);

    let slash_mark = cursor.position();
    cursor.skip_ws();
    let mut bass = None;
    if cursor.try_str("/") {
        cursor.skip_ws();
        bass = cursor.consume_note();
        if bass.is_none() {
            cursor.set_position(slash_mark);
        }
    }

    parse_alterations_and_adds(&mut cursor, &mut alterations, &mut added);

    let tail = cursor.rest().trim().to_string();
    let status = if tail.is_empty() {
        ParseStatus::FullyRecognized
    } else {
        ParseStatus::PartiallyRecognized
    };

    Chord::new(
        Some(root),
        quality,
        seventh,
        extensions,
        alterations,
        suspension,
        added,
        bass,
        original,
        tail,
        status,
    )
}

fn parse_quality(
    cursor: &mut Cursor<'_>,
    quality: &mut Quality,
    seventh: &mut Option<Seventh>,
    extensions: &mut Vec<Extension>,
    alterations: &mut Vec<Alteration>,
) {
    if cursor.try_any_ci(&["halfdim7", "hdim7"]) || cursor.try_str("ø7") {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        push_alt(alterations, Alteration::flat(5));
        return;
    }
    if cursor.try_any_ci(&["halfdim", "hdim"]) || cursor.try_str("ø") {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        push_alt(alterations, Alteration::flat(5));
        return;
    }
    if cursor.try_any_ci(&["dim7"]) || cursor.try_str("°7") || cursor.try_str("o7") {
        *quality = Quality::Diminished;
        *seventh = Some(Seventh::Diminished);
        return;
    }
    if cursor.try_any_ci(&["dim"]) || cursor.try_str("°") || cursor.try_str("o") {
        *quality = Quality::Diminished;
        return;
    }
    if cursor.try_any_ci(&["aug"]) || try_plus_as_augmented(cursor) {
        *quality = Quality::Augmented;
        return;
    }

    if cursor.try_any_ci(&["maj13", "major13"]) || cursor.try_str("Δ13") || cursor.try_str("M13") {
        *seventh = Some(Seventh::Major);
        push_ext(extensions, Extension::Thirteenth);
        return;
    }
    if cursor.try_any_ci(&["maj11", "major11"]) || cursor.try_str("Δ11") || cursor.try_str("M11") {
        *seventh = Some(Seventh::Major);
        push_ext(extensions, Extension::Eleventh);
        return;
    }
    if cursor.try_any_ci(&["maj9", "major9"]) || cursor.try_str("Δ9") || cursor.try_str("M9") {
        *seventh = Some(Seventh::Major);
        push_ext(extensions, Extension::Ninth);
        return;
    }
    if cursor.try_any_ci(&["maj7", "major7"])
        || cursor.try_str("Δ7")
        || cursor.try_str("M7")
        || cursor.try_str("Δ")
    {
        *seventh = Some(Seventh::Major);
        return;
    }
    if cursor.try_any_ci(&["maj", "major"]) || cursor.try_str("M") {
        return;
    }

    if cursor.try_any_ci(&["min13", "minor13"]) {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        push_ext(extensions, Extension::Thirteenth);
        return;
    }
    if cursor.try_any_ci(&["min11", "minor11"]) {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        push_ext(extensions, Extension::Eleventh);
        return;
    }
    if cursor.try_any_ci(&["min9", "minor9"]) {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        push_ext(extensions, Extension::Ninth);
        return;
    }
    if cursor.try_any_ci(&["min7", "minor7"]) || cursor.try_str("-7") {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        return;
    }
    if cursor.try_any_ci(&["min", "minor"]) || cursor.try_str("-") {
        *quality = Quality::Minor;
        return;
    }

    if cursor.try_str("m13") {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        push_ext(extensions, Extension::Thirteenth);
        return;
    }
    if cursor.try_str("m11") {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        push_ext(extensions, Extension::Eleventh);
        return;
    }
    if cursor.try_str("m9") {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        push_ext(extensions, Extension::Ninth);
        return;
    }
    if cursor.try_str("m7") {
        *quality = Quality::Minor;
        *seventh = Some(Seventh::Dominant);
        return;
    }
    if cursor.try_str("m6") {
        *quality = Quality::Minor;
        push_ext(extensions, Extension::Sixth);
        return;
    }
    if cursor.try_str("m") {
        *quality = Quality::Minor;
    }
}

fn parse_extensions(
    cursor: &mut Cursor<'_>,
    quality: Quality,
    seventh: &mut Option<Seventh>,
    extensions: &mut Vec<Extension>,
) {
    if cursor.try_str("13") {
        if seventh.is_none() {
            *seventh = Some(default_seventh(quality));
        }
        push_ext(extensions, Extension::Thirteenth);
    } else if cursor.try_str("11") {
        if seventh.is_none() {
            *seventh = Some(default_seventh(quality));
        }
        push_ext(extensions, Extension::Eleventh);
    } else if cursor.try_str("9") {
        if seventh.is_none() {
            *seventh = Some(default_seventh(quality));
        }
        push_ext(extensions, Extension::Ninth);
    } else if cursor.try_str("7") {
        if seventh.is_none() {
            *seventh = Some(default_seventh(quality));
        }
    } else if cursor.try_str("6") {
        push_ext(extensions, Extension::Sixth);
        if cursor.try_str("9") {
            push_ext(extensions, Extension::Ninth);
        }
    }
}

fn default_seventh(quality: Quality) -> Seventh {
    match quality {
        Quality::Diminished => Seventh::Diminished,
        _ => Seventh::Dominant,
    }
}

fn try_plus_as_augmented(cursor: &mut Cursor<'_>) -> bool {
    let rest = cursor.rest();
    if !rest.starts_with('+') {
        return false;
    }
    // `+5` is a sharp-5 alteration; `+7` / `+9` is augmented plus an extension.
    if let Some(after) = rest.strip_prefix("+5") {
        if after.is_empty() || !after.starts_with(|ch: char| ch.is_ascii_digit()) {
            return false;
        }
    }
    cursor.try_str("+")
}

fn parse_suspension(cursor: &mut Cursor<'_>, suspension: &mut Option<Suspension>) {
    if cursor.try_any_ci(&["sus2"]) {
        *suspension = Some(Suspension::Sus2);
    } else if cursor.try_any_ci(&["sus4", "sus"]) {
        *suspension = Some(Suspension::Sus4);
    }
}

fn parse_alterations_and_adds(
    cursor: &mut Cursor<'_>,
    alterations: &mut Vec<Alteration>,
    added: &mut Vec<AddedTone>,
) {
    let _ = parse_paren_group(cursor, alterations, added);
    while parse_one_alteration(cursor, alterations) || parse_one_add(cursor, added) {}
    let _ = parse_paren_group(cursor, alterations, added);
}

fn parse_paren_group(
    cursor: &mut Cursor<'_>,
    alterations: &mut Vec<Alteration>,
    added: &mut Vec<AddedTone>,
) -> bool {
    if !cursor.try_str("(") {
        return false;
    }
    while parse_one_alteration(cursor, alterations) || parse_one_add(cursor, added) {}
    cursor.try_str(")")
}

fn parse_one_alteration(cursor: &mut Cursor<'_>, alterations: &mut Vec<Alteration>) -> bool {
    const ALTS: &[(&str, u8, i8)] = &[
        ("##13", 13, 2),
        ("##11", 11, 2),
        ("##9", 9, 2),
        ("##5", 5, 2),
        ("#13", 13, 1),
        ("#11", 11, 1),
        ("#9", 9, 1),
        ("#5", 5, 1),
        ("+5", 5, 1),
        ("bb13", 13, -2),
        ("bb9", 9, -2),
        ("bb5", 5, -2),
        ("b13", 13, -1),
        ("b11", 11, -1),
        ("b9", 9, -1),
        ("b6", 6, -1),
        ("b5", 5, -1),
        ("-5", 5, -1),
        ("♭13", 13, -1),
        ("♭9", 9, -1),
        ("♭5", 5, -1),
        ("♯11", 11, 1),
        ("♯9", 9, 1),
        ("♯5", 5, 1),
    ];
    for (token, degree, delta) in ALTS {
        if cursor.try_str(token) {
            push_alt(alterations, Alteration::new(*degree, *delta));
            return true;
        }
    }
    false
}

fn parse_one_add(cursor: &mut Cursor<'_>, added: &mut Vec<AddedTone>) -> bool {
    const ADDS: &[(&str, AddedTone)] = &[
        ("add13", AddedTone::Add13),
        ("add11", AddedTone::Add11),
        ("add9", AddedTone::Add9),
        ("add6", AddedTone::Add6),
        ("add4", AddedTone::Add4),
        ("add2", AddedTone::Add2),
    ];
    for (token, tone) in ADDS {
        if cursor.try_str_ci(token) {
            if !added.contains(tone) {
                added.push(*tone);
            }
            return true;
        }
    }
    false
}

fn push_ext(extensions: &mut Vec<Extension>, ext: Extension) {
    if !extensions.contains(&ext) {
        extensions.push(ext);
    }
}

fn push_alt(alterations: &mut Vec<Alteration>, alt: Alteration) {
    if !alterations.contains(&alt) {
        alterations.push(alt);
    }
}

struct Cursor<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn set_position(&mut self, pos: usize) {
        self.pos = pos.min(self.input.len());
    }

    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn skip_ws(&mut self) {
        let rest = self.rest();
        let trimmed = rest.trim_start();
        self.pos += rest.len() - trimmed.len();
    }

    fn try_str(&mut self, token: &str) -> bool {
        if self.rest().starts_with(token) {
            self.pos += token.len();
            true
        } else {
            false
        }
    }

    fn try_str_ci(&mut self, token: &str) -> bool {
        let rest = self.rest();
        if rest.len() >= token.len() && rest[..token.len()].eq_ignore_ascii_case(token) {
            self.pos += token.len();
            true
        } else {
            false
        }
    }

    fn try_any_ci(&mut self, tokens: &[&str]) -> bool {
        tokens.iter().any(|token| self.try_str_ci(token))
    }

    fn consume_note(&mut self) -> Option<Note> {
        let (note, rest) = Note::consume(self.rest())?;
        self.pos += self.rest().len() - rest.len();
        Some(note)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chord::{ParseStatus, Quality, Seventh};

    fn assert_full(input: &str, symbol: &str) {
        let chord = parse_chord(input);
        assert_eq!(
            chord.status(),
            ParseStatus::FullyRecognized,
            "{input} -> {} tail={}",
            chord.symbol(),
            chord.unparsed_tail()
        );
        assert_eq!(chord.symbol(), symbol, "{input}");
    }

    #[test]
    fn parses_required_symbol_families() {
        let cases = [
            ("C", "C"),
            ("Cm", "Cm"),
            ("C#", "C#"),
            ("Db", "Db"),
            ("F#m", "F#m"),
            ("C7", "C7"),
            ("Cm7", "Cm7"),
            ("Cmaj7", "Cmaj7"),
            ("C9", "C9"),
            ("C11", "C11"),
            ("C13", "C13"),
            ("C6", "C6"),
            ("Cm6", "Cm6"),
            ("Cadd9", "Cadd9"),
            ("C7b5", "C7b5"),
            ("C7#5", "C7#5"),
            ("C7b9", "C7b9"),
            ("C7#9", "C7#9"),
            ("Cmaj7#11", "Cmaj7#11"),
            ("Csus2", "Csus2"),
            ("Csus4", "Csus4"),
            ("Cdim", "Cdim"),
            ("Caug", "Caug"),
            ("G/B", "G/B"),
            ("D/F#", "D/F#"),
            ("F#m7b5/C#", "F#m7b5/C#"),
        ];
        for (input, symbol) in cases {
            assert_full(input, symbol);
        }
    }

    #[test]
    fn parses_slash_with_spaces_and_parentheses() {
        assert_full("G / B", "G/B");
        assert_full("C7(b9)", "C7b9");
        assert_full("Cmaj7(#11)", "Cmaj7#11");
        assert_full("C(add9)", "Cadd9");
    }

    #[test]
    fn minor_is_not_confused_with_major_m_token() {
        let cm7 = parse_chord("Cm7");
        assert_eq!(cm7.quality(), Quality::Minor);
        assert_eq!(cm7.seventh(), Some(Seventh::Dominant));
        assert_eq!(cm7.symbol(), "Cm7");

        let cmaj7 = parse_chord("CM7");
        assert_eq!(cmaj7.quality(), Quality::Major);
        assert_eq!(cmaj7.seventh(), Some(Seventh::Major));
        assert_eq!(cmaj7.symbol(), "Cmaj7");
    }

    #[test]
    fn half_diminished_canonicalizes_to_m7b5() {
        let chord = parse_chord("Cø");
        assert_eq!(chord.status(), ParseStatus::FullyRecognized);
        assert_eq!(chord.quality(), Quality::Minor);
        assert_eq!(chord.seventh(), Some(Seventh::Dominant));
        assert_eq!(chord.symbol(), "Cm7b5");
    }

    #[test]
    fn unrecognized_and_partial_cases() {
        let unknown = ["", "   ", "Hello", "H7", "maj7", "1", "N.C.", "%", "/G"];
        for input in unknown {
            let chord = parse_chord(input);
            assert_eq!(
                chord.status(),
                ParseStatus::Unrecognized,
                "{input:?} was {:?}",
                chord.status()
            );
            assert_eq!(chord.source_text().trim(), input.trim());
        }

        let partial = parse_chord("Cmaj7wow");
        assert_eq!(partial.status(), ParseStatus::PartiallyRecognized);
        assert_eq!(partial.symbol(), "Cmaj7");
        assert_eq!(partial.unparsed_tail(), "wow");
        assert_eq!(partial.source_text(), "Cmaj7wow");

        let cfoo = parse_chord("Cxyz");
        assert_eq!(cfoo.status(), ParseStatus::PartiallyRecognized);
        assert_eq!(cfoo.symbol(), "C");
    }

    #[test]
    fn dim7_and_aug7_and_sus_defaults() {
        assert_full("Cdim7", "Cdim7");
        assert_full("C+7", "Caug7");
        assert_full("Csus", "Csus4");
        assert_full("C7sus4", "C7sus4");
        assert_full("C+5", "C#5");
    }
}
