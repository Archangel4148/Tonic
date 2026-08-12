import type { ImportFormat } from "./types";

export const SAMPLE_CHORDPRO = `{title: Amazing Grace}
{artist: Traditional}
{key: G}
{tempo: 72}
{time: 3/4}

{start_of_verse: Verse 1}
[G]Amazing grace, how [D]sweet the sound
That [Em]saved a wretch like [C]me
{end_of_verse}

{start_of_chorus}
[G]Amazing grace
{end_of_chorus}
`;

export const SAMPLE_PLAIN = `Title: Amazing Grace
Artist: Traditional
Key: G

Verse 1
C          G
Amazing grace how sweet
F             C
The sound that saved
`;

export const SAMPLE_UNKNOWN = `{title: Jazz Sketch}
{key: C}

[C]Hello [Xyz]world [G]there
[F#m7b5/C#]keep this slash
`;

export const SAMPLE_MUSICXML = `<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <work>
    <work-title>Twinkle</work-title>
  </work>
  <identification>
    <creator type="composer">Traditional</creator>
  </identification>
  <part-list>
    <score-part id="P1">
      <part-name>Voice</part-name>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key>
          <fifths>0</fifths>
          <mode>major</mode>
        </key>
        <time>
          <beats>4</beats>
          <beat-type>4</beat-type>
        </time>
        <clef>
          <sign>G</sign>
          <line>2</line>
        </clef>
      </attributes>
      <harmony>
        <root>
          <root-step>C</root-step>
        </root>
        <kind>major</kind>
      </harmony>
      <note>
        <pitch>
          <step>C</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
        <type>quarter</type>
        <lyric>
          <text>Twin</text>
        </lyric>
      </note>
      <note>
        <pitch>
          <step>G</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
        <type>quarter</type>
        <lyric>
          <text>kle</text>
        </lyric>
      </note>
    </measure>
  </part>
</score-partwise>
`;

export const SAMPLES: ReadonlyArray<{
  id: string;
  label: string;
  text: string;
  format: ImportFormat;
}> = [
  {
    id: "chordpro",
    label: "Amazing Grace (ChordPro)",
    text: SAMPLE_CHORDPRO,
    format: "chordPro",
  },
  {
    id: "plain",
    label: "Amazing Grace (plain text)",
    text: SAMPLE_PLAIN,
    format: "plainText",
  },
  {
    id: "unknown",
    label: "Unknown chords",
    text: SAMPLE_UNKNOWN,
    format: "chordPro",
  },
  {
    id: "musicxml",
    label: "Twinkle (MusicXML)",
    text: SAMPLE_MUSICXML,
    format: "musicXml",
  },
];
