# Music theory engine

The engine lives in `tonic-domain`. It has **no UI, Tauri, or persistence dependencies**. There is one transposition implementation.

Phase 2 covers notes, keys, chord parsing, transposition, and a small capo helper. The canonical song model is still Phase 3.

## Public entry points

| API                                       | Purpose                                              |
| ----------------------------------------- | ---------------------------------------------------- |
| `Note::parse` / `Note::consume`           | Spelled notes (`C`, `F#`, `Bb`, `E#`, `Cb`, …)       |
| `PitchClass` / `Semitones`                | Pitch class `0..12` and signed intervals             |
| `Key::parse`                              | Major/minor keys (`C`, `Am`, `F# minor`, `Bb major`) |
| `Key::transpose_semitones`                | Move a key by ±N using common tonic spellings        |
| `Key::spell`                              | Enharmonic spelling in a key                         |
| `parse_chord`                             | Structured chord parse; never discards text          |
| `transpose_semitones`                     | Offset transpose, keep accidental family             |
| `transpose_to_key`                        | Source key → target key, spell in the target key     |
| `Capo` / `played_shape` / `concert_pitch` | Capo vs sounding pitch                               |

## Chord model

A chord is **not** an opaque string. Components:

- Root note (optional if unrecognized)
- Quality: major, minor, diminished, augmented
- Seventh: dominant (`b7`), major, diminished
- Extensions: 6, 9, 11, 13
- Alterations: degree + accidental (`b5`, `#11`, …)
- Suspension: sus2 / sus4
- Added tones: `add9`, `add4`, …
- Bass note (slash chords)
- Original source text
- Parse status and any unparsed tail

`Chord::symbol()` renders a canonical ASCII symbol. Unicode input (`♯`, `♭`, `ø`, `Δ`) is accepted on parse.

## Parse status

| Status               | Meaning                                         |
| -------------------- | ----------------------------------------------- |
| Fully recognized     | Entire symbol consumed                          |
| Partially recognized | Root and some suffix parsed; leftover text kept |
| Unrecognized         | No root; original text preserved                |

`Hello`, `N.C.`, `H7`, and empty input are unrecognized. `Cmaj7wow` is partial: symbol `Cmaj7`, tail `wow`.

The parser is a token scanner (longest-match quality/extension/alteration tokens). It is not a dictionary of full chord strings.

## Spelling strategy

Documented and tested:

1. **Keyless semitone transpose** (`transpose_semitones`): move pitch class, then respell by accidental family.
   - Flat notes stay on the flat chromatic: `Bb + 2 → C`, `Eb + 1 → E`
   - Sharp or natural notes use the sharp chromatic: `F# + 2 → G#`, `B + 2 → C#`, `C + 1 → C#`
2. **Key context** (`Key::spell` / `transpose_to_key`):
   - If the pitch class is a diatonic scale degree of the destination key, use that spelling.
   - Otherwise use the key signature’s accidental family (sharps vs flats).
   - Example: in Ab major, pitch class `8` is **Ab**, not G#.

`C# major` and `Db major` are different keys: same tonic pitch class, different spelling families.

Natural minor is used for minor-key diatonic spellings. Relative major/minor therefore share accidental preference (A minor with C major, D minor with F major, …).

## Capo

The engine distinguishes:

- Concert / sounding pitch
- Played shape
- Capo fret (`0` = none, max `12`)

Example: sounding **A**, capo **2**, played shapes **G**.

```text
played_shape(A, capo 2) = G
concert_pitch(G, capo 2) = A
```

## Canonical symbol conventions

- Major quality is unmarked: `C`, not `Cmaj` (unless there is a major seventh: `Cmaj7`)
- Minor is `m`: `Cm`, `Cm7`
- Half-diminished `ø` / `hdim` becomes `m7b5`
- Bare `sus` means `sus4`
- `CM7` is major seven; `Cm7` is minor seven (`M` vs `m` is case-sensitive)
- Parentheses around alterations/adds are accepted: `C7(b9)`, `C(add9)`

## Out of scope here

- Song / line / section model (Phase 3)
- Transpose UI or Tauri commands (see Phase 5 viewer)
- Changing chord quality when moving between major and minor keys — only pitch-bearing notes change
