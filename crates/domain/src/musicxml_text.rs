//! Preserve original MusicXML engraving while rewriting sounding pitches.

use crate::key::Key;
use crate::note::{Accidental, Letter, Note, Spelling};
use crate::score::ScorePitch;

/// Transpose `<pitch>`, `<key>` fifths, and harmony `<root>` in an existing document.
///
/// Layout, staves, clefs, beams, stems, dynamics, and directions are copied unchanged.
#[must_use]
pub fn transpose_musicxml_text(xml: &str, semitones: i32, spelling: Spelling) -> String {
    if semitones == 0 {
        return xml.to_string();
    }
    let with_pitches = rewrite_elements(xml, "pitch", |inner| {
        rewrite_pitch(inner, semitones, spelling)
    });
    let with_keys = rewrite_elements(&with_pitches, "key", |inner| rewrite_key(inner, semitones));
    rewrite_elements(&with_keys, "root", |inner| {
        rewrite_root(inner, semitones, spelling)
    })
}

fn rewrite_pitch(inner: &str, semitones: i32, spelling: Spelling) -> Option<String> {
    let step = tag_text(inner, "step")?;
    let letter = Letter::from_char(step.chars().next()?)?;
    let alter = tag_text(inner, "alter")
        .and_then(|text| text.parse::<i32>().ok())
        .unwrap_or(0);
    let accidental = Accidental::from_semitones(alter).unwrap_or(Accidental::Natural);
    let octave = tag_text(inner, "octave")?.parse::<i8>().ok()?;
    let transposed =
        ScorePitch::new(Note::new(letter, accidental), octave).transpose(semitones, spelling);
    let alter_semitones = transposed.note.accidental().semitones();
    let mut out = replace_tag_text(
        inner,
        "step",
        &transposed.note.letter().as_char().to_string(),
    );
    out = if inner.contains("<alter") {
        if alter_semitones == 0 {
            remove_tag(&out, "alter")
        } else {
            replace_tag_text(&out, "alter", &alter_semitones.to_string())
        }
    } else if alter_semitones != 0 {
        insert_after_tag(&out, "step", &format!("<alter>{alter_semitones}</alter>"))
    } else {
        out
    };
    Some(replace_tag_text(
        &out,
        "octave",
        &transposed.octave.to_string(),
    ))
}

fn rewrite_key(inner: &str, semitones: i32) -> Option<String> {
    let fifths = tag_text(inner, "fifths")?.parse::<i32>().ok()?;
    let mode = tag_text(inner, "mode").unwrap_or_default();
    let minor = mode.eq_ignore_ascii_case("minor");
    let next = Key::from_fifths(fifths, minor)?.transpose_semitones(semitones);
    let new_fifths = next.fifths()?;
    Some(replace_tag_text(inner, "fifths", &new_fifths.to_string()))
}

fn rewrite_root(inner: &str, semitones: i32, spelling: Spelling) -> Option<String> {
    let step = tag_text(inner, "root-step")?;
    let letter = Letter::from_char(step.chars().next()?)?;
    let alter = tag_text(inner, "root-alter")
        .and_then(|text| text.parse::<i32>().ok())
        .unwrap_or(0);
    let accidental = Accidental::from_semitones(alter).unwrap_or(Accidental::Natural);
    let transposed =
        Note::new(letter, accidental).transpose(crate::pitch::Semitones::new(semitones), spelling);
    let alter_semitones = transposed.accidental().semitones();
    let mut out = replace_tag_text(
        inner,
        "root-step",
        &transposed.letter().as_char().to_string(),
    );
    out = if inner.contains("<root-alter") {
        if alter_semitones == 0 {
            remove_tag(&out, "root-alter")
        } else {
            replace_tag_text(&out, "root-alter", &alter_semitones.to_string())
        }
    } else if alter_semitones != 0 {
        insert_after_tag(
            &out,
            "root-step",
            &format!("<root-alter>{alter_semitones}</root-alter>"),
        )
    } else {
        out
    };
    Some(out)
}

fn rewrite_elements(
    xml: &str,
    name: &str,
    mut rewrite: impl FnMut(&str) -> Option<String>,
) -> String {
    let open = format!("<{name}");
    let close = format!("</{name}>");
    let mut rest = xml;
    let mut out = String::with_capacity(xml.len());
    while let Some(rel) = find_open_tag(rest, &open) {
        out.push_str(&rest[..rel]);
        let after_name = rel + open.len();
        let Some(gt) = rest[after_name..].find('>') else {
            out.push_str(&rest[rel..]);
            return out;
        };
        let inner_start = after_name + gt + 1;
        let Some(inner_end_rel) = rest[inner_start..].find(&close) else {
            out.push_str(&rest[rel..]);
            return out;
        };
        let inner_end = inner_start + inner_end_rel;
        let open_tag = &rest[rel..inner_start];
        let inner = &rest[inner_start..inner_end];
        out.push_str(open_tag);
        match rewrite(inner) {
            Some(next) => out.push_str(&next),
            None => out.push_str(inner),
        }
        out.push_str(&close);
        rest = &rest[inner_end + close.len()..];
    }
    out.push_str(rest);
    out
}

fn find_open_tag(xml: &str, open: &str) -> Option<usize> {
    let mut offset = 0;
    let mut search = xml;
    while let Some(rel) = search.find(open) {
        let abs = offset + rel;
        let next = xml[abs + open.len()..].chars().next();
        if matches!(next, Some('>' | ' ' | '\n' | '\r' | '\t' | '/')) {
            return Some(abs);
        }
        offset = abs + open.len();
        search = &xml[offset..];
    }
    None
}

fn tag_text<'a>(inner: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}");
    let rel = find_open_tag(inner, &open)?;
    let after_name = rel + open.len();
    let gt = inner[after_name..].find('>')?;
    let start = after_name + gt + 1;
    let close = format!("</{name}>");
    let end_rel = inner[start..].find(&close)?;
    Some(inner[start..start + end_rel].trim())
}

fn replace_tag_text(inner: &str, name: &str, value: &str) -> String {
    let open = format!("<{name}");
    let Some(rel) = find_open_tag(inner, &open) else {
        return inner.to_string();
    };
    let after_name = rel + open.len();
    let Some(gt) = inner[after_name..].find('>') else {
        return inner.to_string();
    };
    let start = after_name + gt + 1;
    let close = format!("</{name}>");
    let Some(end_rel) = inner[start..].find(&close) else {
        return inner.to_string();
    };
    let mut out = String::with_capacity(inner.len() + value.len());
    out.push_str(&inner[..start]);
    out.push_str(value);
    out.push_str(&inner[start + end_rel..]);
    out
}

fn remove_tag(inner: &str, name: &str) -> String {
    let open = format!("<{name}");
    let Some(rel) = find_open_tag(inner, &open) else {
        return inner.to_string();
    };
    let close = format!("</{name}>");
    let Some(end_rel) = inner[rel..].find(&close) else {
        return inner.to_string();
    };
    let end = rel + end_rel + close.len();
    let mut start = rel;
    while start > 0 && inner.as_bytes()[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    let mut out = String::with_capacity(inner.len());
    out.push_str(&inner[..start]);
    out.push_str(&inner[end..]);
    out
}

fn insert_after_tag(inner: &str, name: &str, fragment: &str) -> String {
    let close = format!("</{name}>");
    let Some(end_rel) = inner.find(&close) else {
        return inner.to_string();
    };
    let end = end_rel + close.len();
    let indent = leading_indent_before(inner, inner.find(&format!("<{name}")).unwrap_or(0));
    let mut out = String::with_capacity(inner.len() + fragment.len() + indent.len() + 1);
    out.push_str(&inner[..end]);
    out.push('\n');
    out.push_str(indent);
    out.push_str(fragment);
    out.push_str(&inner[end..]);
    out
}

fn leading_indent_before(inner: &str, index: usize) -> &str {
    let line_start = inner[..index].rfind('\n').map_or(0, |i| i + 1);
    let indent_end = inner[line_start..index]
        .chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .count();
    &inner[line_start..line_start + indent_end]
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIANO: &str = r#"<score-partwise>
  <part id="P1">
    <measure number="1">
      <attributes>
        <staves>2</staves>
        <clef number="1"><sign>G</sign><line>2</line></clef>
        <clef number="2"><sign>F</sign><line>4</line></clef>
      </attributes>
      <note>
        <pitch>
          <step>E</step>
          <octave>5</octave>
        </pitch>
        <staff>1</staff>
      </note>
      <backup><duration>1</duration></backup>
      <note>
        <pitch>
          <step>A</step>
          <octave>2</octave>
        </pitch>
        <staff>2</staff>
      </note>
    </measure>
  </part>
</score-partwise>
"#;

    #[test]
    fn zero_steps_keeps_original_bytes() {
        assert_eq!(
            transpose_musicxml_text(PIANO, 0, Spelling::PreserveAccidentalFamily),
            PIANO
        );
    }

    #[test]
    fn transpose_keeps_grand_staff_and_moves_pitches() {
        let out = transpose_musicxml_text(PIANO, 2, Spelling::InKey(Key::parse("D").unwrap()));
        assert!(out.contains("<staves>2</staves>"), "{out}");
        assert!(out.contains("<clef number=\"1\">"), "{out}");
        assert!(out.contains("<clef number=\"2\">"), "{out}");
        assert!(out.contains("<sign>G</sign>"), "{out}");
        assert!(out.contains("<sign>F</sign>"), "{out}");
        assert!(out.contains("<step>F</step>"), "{out}");
        assert!(out.contains("<alter>1</alter>"), "{out}");
        assert!(out.contains("<octave>5</octave>"), "{out}");
        assert!(out.contains("<step>B</step>"), "{out}");
        assert!(!out.contains("<step>E</step>"), "{out}");
        assert!(!out.contains("<step>A</step>"), "{out}");
    }
}
