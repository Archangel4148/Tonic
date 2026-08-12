//! Canonical sheet-music score. Distinct from the chord-chart [`Song`] body.
//!
//! Written pitches are authoritative. Display MusicXML is derived when the
//! performance key differs from the original key.

use serde::{Deserialize, Serialize};

use crate::chord::Chord;
use crate::key::Key;
use crate::note::{Accidental, Letter, Note, Spelling};
use crate::parse::parse_chord;
use crate::pitch::Semitones;
use crate::song::TimeSignature;
use crate::transpose::transpose_to_key;

/// Pitch with MusicXML octave (`4` = the octave containing middle C).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScorePitch {
    pub note: Note,
    pub octave: i8,
}

impl ScorePitch {
    #[must_use]
    pub fn new(note: Note, octave: i8) -> Self {
        Self { note, octave }
    }

    #[must_use]
    pub fn midi(self) -> i32 {
        i32::from(self.octave + 1) * 12 + i32::from(self.note.pitch_class().value())
    }

    #[must_use]
    pub fn transpose(self, semitones: i32, spelling: Spelling) -> Self {
        let midi = self.midi() + semitones;
        let note = self.note.transpose(Semitones::new(semitones), spelling);
        let octave = (midi.div_euclid(12) - 1) as i8;
        Self { note, octave }
    }
}

/// Staff clef used for engraving.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Clef {
    Treble,
    Bass,
    Alto,
    Tenor,
    Percussion,
    Other(String),
}

impl Clef {
    #[must_use]
    pub fn from_musicxml(sign: &str, line: Option<u8>) -> Self {
        match (sign.trim().to_ascii_uppercase().as_str(), line) {
            ("G", None | Some(2)) => Self::Treble,
            ("F", None | Some(4)) => Self::Bass,
            ("C", Some(3)) => Self::Alto,
            ("C", Some(4)) => Self::Tenor,
            ("percussion", _) | ("PERCUSSION", _) => Self::Percussion,
            _ => Self::Other(sign.to_string()),
        }
    }

    #[must_use]
    pub fn musicxml_sign(&self) -> &'static str {
        match self {
            Self::Treble => "G",
            Self::Bass => "F",
            Self::Alto | Self::Tenor => "C",
            Self::Percussion => "percussion",
            Self::Other(_) => "G",
        }
    }

    #[must_use]
    pub fn musicxml_line(&self) -> u8 {
        match self {
            Self::Treble | Self::Other(_) => 2,
            Self::Bass => 4,
            Self::Alto => 3,
            Self::Tenor => 4,
            Self::Percussion => 3,
        }
    }
}

/// Key, time, clef, and divisions active from this measure onward when present.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasureAttributes {
    #[serde(default)]
    pub divisions: Option<u32>,
    #[serde(default)]
    pub key: Option<Key>,
    #[serde(default)]
    pub time: Option<TimeSignature>,
    #[serde(default)]
    pub clef: Option<Clef>,
}

/// A pitched note in a measure.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreNote {
    pub pitch: ScorePitch,
    pub duration: u32,
    #[serde(default)]
    pub voice: u8,
    #[serde(default)]
    pub staff: u8,
    #[serde(default)]
    pub chord: bool,
    #[serde(default)]
    pub lyric: Option<String>,
    #[serde(default)]
    pub note_type: Option<String>,
}

/// A rest in a measure.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreRest {
    pub duration: u32,
    #[serde(default)]
    pub voice: u8,
    #[serde(default)]
    pub staff: u8,
    #[serde(default)]
    pub note_type: Option<String>,
}

/// Chord symbol attached to the score (MusicXML `<harmony>`).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreHarmony {
    pub symbol: String,
    #[serde(default)]
    pub chord: Option<Chord>,
}

/// One timed or structural event inside a measure.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MeasureEvent {
    Note(ScoreNote),
    Rest(ScoreRest),
    Harmony(ScoreHarmony),
    Backup(u32),
    Forward(u32),
}

/// One measure of one part.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Measure {
    pub number: u32,
    #[serde(default)]
    pub attributes: Option<MeasureAttributes>,
    #[serde(default)]
    pub events: Vec<MeasureEvent>,
}

/// One instrument/staff part.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScorePart {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub measures: Vec<Measure>,
}

/// Structured score. Not a chord-chart `Song` body.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    #[serde(default)]
    pub work_title: Option<String>,
    #[serde(default)]
    pub parts: Vec<ScorePart>,
}

impl Score {
    #[must_use]
    pub fn transpose_semitones(&self, semitones: i32, spelling: Spelling) -> Self {
        let mut out = self.clone();
        for part in &mut out.parts {
            for measure in &mut part.measures {
                if let Some(attributes) = measure.attributes.as_mut() {
                    if let Some(key) = attributes.key.as_mut() {
                        *key = key.transpose_semitones(semitones);
                    }
                }
                for event in &mut measure.events {
                    match event {
                        MeasureEvent::Note(note) => {
                            note.pitch = note.pitch.transpose(semitones, spelling);
                        }
                        MeasureEvent::Harmony(harmony) => {
                            if let Some(chord) = harmony.chord.as_mut() {
                                *chord = match spelling {
                                    Spelling::InKey(key) => {
                                        let from = key.transpose_semitones(-semitones);
                                        transpose_to_key(chord, from, key)
                                    }
                                    Spelling::PreserveAccidentalFamily => {
                                        crate::transpose::transpose_semitones(chord, semitones)
                                    }
                                };
                                harmony.symbol = chord.symbol();
                            } else if !harmony.symbol.is_empty() {
                                let parsed = parse_chord(&harmony.symbol);
                                if parsed.root().is_some() {
                                    let transposed =
                                        crate::transpose::transpose_semitones(&parsed, semitones);
                                    harmony.symbol = transposed.symbol();
                                    harmony.chord = Some(transposed);
                                }
                            }
                        }
                        MeasureEvent::Rest(_)
                        | MeasureEvent::Backup(_)
                        | MeasureEvent::Forward(_) => {}
                    }
                }
            }
        }
        out
    }

    /// Emit partwise MusicXML for a notation renderer.
    #[must_use]
    pub fn to_musicxml(&self) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 4.0 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">
<score-partwise version="4.0">
"#,
        );
        if let Some(title) = &self.work_title {
            xml.push_str(&format!(
                "  <work>\n    <work-title>{}</work-title>\n  </work>\n",
                xml_escape(title)
            ));
        }
        xml.push_str("  <part-list>\n");
        for part in &self.parts {
            xml.push_str(&format!(
                "    <score-part id=\"{}\">\n      <part-name>{}</part-name>\n    </score-part>\n",
                xml_escape(&part.id),
                xml_escape(&part.name)
            ));
        }
        xml.push_str("  </part-list>\n");
        for part in &self.parts {
            xml.push_str(&format!("  <part id=\"{}\">\n", xml_escape(&part.id)));
            let mut divisions = 1u32;
            for measure in &part.measures {
                if let Some(div) = measure
                    .attributes
                    .as_ref()
                    .and_then(|attributes| attributes.divisions)
                {
                    divisions = div.max(1);
                }
                xml.push_str(&format!("    <measure number=\"{}\">\n", measure.number));
                if let Some(attributes) = &measure.attributes {
                    xml.push_str("      <attributes>\n");
                    if let Some(div) = attributes.divisions {
                        xml.push_str(&format!("        <divisions>{div}</divisions>\n"));
                    }
                    if let Some(key) = attributes.key {
                        if let Some(fifths) = key.fifths() {
                            xml.push_str("        <key>\n");
                            xml.push_str(&format!("          <fifths>{fifths}</fifths>\n"));
                            let mode = match key.mode() {
                                crate::key::Mode::Minor => "minor",
                                crate::key::Mode::Major => "major",
                            };
                            xml.push_str(&format!("          <mode>{mode}</mode>\n"));
                            xml.push_str("        </key>\n");
                        }
                    }
                    if let Some(time) = attributes.time {
                        xml.push_str("        <time>\n");
                        xml.push_str(&format!("          <beats>{}</beats>\n", time.numerator()));
                        xml.push_str(&format!(
                            "          <beat-type>{}</beat-type>\n",
                            time.denominator()
                        ));
                        xml.push_str("        </time>\n");
                    }
                    if let Some(clef) = &attributes.clef {
                        xml.push_str("        <clef>\n");
                        xml.push_str(&format!(
                            "          <sign>{}</sign>\n",
                            clef.musicxml_sign()
                        ));
                        xml.push_str(&format!(
                            "          <line>{}</line>\n",
                            clef.musicxml_line()
                        ));
                        xml.push_str("        </clef>\n");
                    }
                    xml.push_str("      </attributes>\n");
                }
                for event in &measure.events {
                    match event {
                        MeasureEvent::Harmony(harmony) => {
                            xml.push_str(&harmony_xml(harmony));
                        }
                        MeasureEvent::Note(note) => {
                            xml.push_str(&note_xml(note, divisions));
                        }
                        MeasureEvent::Rest(rest) => {
                            xml.push_str(&rest_xml(rest, divisions));
                        }
                        MeasureEvent::Backup(duration) => {
                            xml.push_str(&format!(
                                "      <backup>\n        <duration>{duration}</duration>\n      </backup>\n"
                            ));
                        }
                        MeasureEvent::Forward(duration) => {
                            xml.push_str(&format!(
                                "      <forward>\n        <duration>{duration}</duration>\n      </forward>\n"
                            ));
                        }
                    }
                }
                xml.push_str("    </measure>\n");
            }
            xml.push_str("  </part>\n");
        }
        xml.push_str("</score-partwise>\n");
        xml
    }
}

fn note_xml(note: &ScoreNote, divisions: u32) -> String {
    let mut xml = String::from("      <note>\n");
    if note.chord {
        xml.push_str("        <chord/>\n");
    }
    xml.push_str(&pitch_xml(note.pitch));
    xml.push_str(&format!("        <duration>{}</duration>\n", note.duration));
    if note.voice > 0 {
        xml.push_str(&format!("        <voice>{}</voice>\n", note.voice));
    }
    let note_type = note
        .note_type
        .clone()
        .unwrap_or_else(|| infer_note_type(note.duration, divisions).to_string());
    xml.push_str(&format!(
        "        <type>{}</type>\n",
        xml_escape(&note_type)
    ));
    if note.staff > 0 {
        xml.push_str(&format!("        <staff>{}</staff>\n", note.staff));
    }
    if let Some(lyric) = &note.lyric {
        xml.push_str(&format!(
            "        <lyric number=\"1\">\n          <syllabic>single</syllabic>\n          <text>{}</text>\n        </lyric>\n",
            xml_escape(lyric)
        ));
    }
    xml.push_str("      </note>\n");
    xml
}

fn rest_xml(rest: &ScoreRest, divisions: u32) -> String {
    let note_type = rest
        .note_type
        .clone()
        .unwrap_or_else(|| infer_note_type(rest.duration, divisions).to_string());
    let mut xml = String::from("      <note>\n        <rest/>\n");
    xml.push_str(&format!("        <duration>{}</duration>\n", rest.duration));
    if rest.voice > 0 {
        xml.push_str(&format!("        <voice>{}</voice>\n", rest.voice));
    }
    xml.push_str(&format!(
        "        <type>{}</type>\n",
        xml_escape(&note_type)
    ));
    xml.push_str("      </note>\n");
    xml
}

fn pitch_xml(pitch: ScorePitch) -> String {
    let alter = match pitch.note.accidental() {
        Accidental::DoubleFlat => Some(-2),
        Accidental::Flat => Some(-1),
        Accidental::Natural => None,
        Accidental::Sharp => Some(1),
        Accidental::DoubleSharp => Some(2),
    };
    let mut xml = String::from("        <pitch>\n");
    xml.push_str(&format!(
        "          <step>{}</step>\n",
        pitch.note.letter().as_char()
    ));
    if let Some(alter) = alter {
        xml.push_str(&format!("          <alter>{alter}</alter>\n"));
    }
    xml.push_str(&format!("          <octave>{}</octave>\n", pitch.octave));
    xml.push_str("        </pitch>\n");
    xml
}

fn harmony_xml(harmony: &ScoreHarmony) -> String {
    let parsed = harmony
        .chord
        .clone()
        .unwrap_or_else(|| parse_chord(&harmony.symbol));
    let root = parsed.root().unwrap_or_else(|| Note::natural(Letter::C));
    let kind = musicxml_kind(&parsed);
    let alter = match root.accidental() {
        Accidental::DoubleFlat => Some(-2),
        Accidental::Flat => Some(-1),
        Accidental::Natural => None,
        Accidental::Sharp => Some(1),
        Accidental::DoubleSharp => Some(2),
    };
    let mut xml = String::from("      <harmony>\n        <root>\n");
    xml.push_str(&format!(
        "          <root-step>{}</root-step>\n",
        root.letter().as_char()
    ));
    if let Some(alter) = alter {
        xml.push_str(&format!("          <root-alter>{alter}</root-alter>\n"));
    }
    xml.push_str("        </root>\n");
    xml.push_str(&format!("        <kind>{kind}</kind>\n"));
    xml.push_str("      </harmony>\n");
    xml
}

fn musicxml_kind(chord: &Chord) -> &'static str {
    if chord.root().is_none() {
        return "none";
    }
    match (chord.quality(), chord.seventh()) {
        (crate::chord::Quality::Diminished, Some(crate::chord::Seventh::Dominant)) => {
            "half-diminished"
        }
        (crate::chord::Quality::Diminished, _) => "diminished",
        (crate::chord::Quality::Augmented, _) => "augmented",
        (crate::chord::Quality::Minor, Some(crate::chord::Seventh::Dominant)) => "minor-seventh",
        (crate::chord::Quality::Minor, Some(crate::chord::Seventh::Major)) => "major-minor",
        (crate::chord::Quality::Minor, _) => "minor",
        (crate::chord::Quality::Major, Some(crate::chord::Seventh::Major)) => "major-seventh",
        (crate::chord::Quality::Major, Some(crate::chord::Seventh::Dominant)) => "dominant",
        (crate::chord::Quality::Major, _) => "major",
    }
}

fn infer_note_type(duration: u32, divisions: u32) -> &'static str {
    if divisions == 0 {
        return "quarter";
    }
    let quarters = f64::from(duration) / f64::from(divisions);
    if quarters >= 3.5 {
        "whole"
    } else if quarters >= 1.75 {
        "half"
    } else if quarters >= 0.875 {
        "quarter"
    } else if quarters >= 0.4 {
        "eighth"
    } else if quarters >= 0.2 {
        "16th"
    } else {
        "32nd"
    }
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c4_quarter() -> ScoreNote {
        ScoreNote {
            pitch: ScorePitch::new(Note::natural(Letter::C), 4),
            duration: 1,
            voice: 1,
            staff: 1,
            chord: false,
            lyric: Some("Hi".into()),
            note_type: Some("quarter".into()),
        }
    }

    #[test]
    fn transpose_moves_written_pitch() {
        let score = Score {
            work_title: Some("Demo".into()),
            parts: vec![ScorePart {
                id: "P1".into(),
                name: "Music".into(),
                measures: vec![Measure {
                    number: 1,
                    attributes: Some(MeasureAttributes {
                        divisions: Some(1),
                        key: Key::parse("C"),
                        time: TimeSignature::new(4, 4),
                        clef: Some(Clef::Treble),
                    }),
                    events: vec![MeasureEvent::Note(c4_quarter())],
                }],
            }],
        };
        let transposed = score.transpose_semitones(2, Spelling::InKey(Key::parse("D").unwrap()));
        match &transposed.parts[0].measures[0].events[0] {
            MeasureEvent::Note(note) => {
                assert_eq!(note.pitch.note.symbol(), "D");
                assert_eq!(note.pitch.octave, 4);
            }
            _ => panic!("expected note"),
        }
        assert_eq!(
            transposed.parts[0].measures[0]
                .attributes
                .as_ref()
                .unwrap()
                .key
                .unwrap()
                .symbol(),
            "D"
        );
        let xml = transposed.to_musicxml();
        assert!(xml.contains("<step>D</step>"), "{xml}");
        assert!(!xml.contains("<step>C</step>"), "{xml}");
        assert!(score.to_musicxml().contains("<step>C</step>"));
    }
}
