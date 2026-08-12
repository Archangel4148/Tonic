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

export const SAMPLES = [
  { id: "chordpro", label: "Amazing Grace (ChordPro)", text: SAMPLE_CHORDPRO },
  { id: "plain", label: "Amazing Grace (plain text)", text: SAMPLE_PLAIN },
  { id: "unknown", label: "Unknown chords", text: SAMPLE_UNKNOWN },
] as const;
