# Import

ChordPro and plain-text parsers live in `tonic-import`. They produce the canonical [`Song`](./song-model.md) model. They do not own music theory, UI, or persistence.

Application code calls `AppServices::import_song` (or `tonic_import::import` / `import_auto`). Phase 4 does not expose import over IPC or in the React shell.

## API

```text
import(input, ImportFormat, id) → ImportResult { song, warnings }
import_auto(input, id)          → same, after detect_format(input)
format_from_extension(ext)      → Option<ImportFormat>
```

`ImportResult` never hard-fails. Usable chords, lyrics, and metadata are always kept. If any warning was produced, `summary_message()` is:

> Some content could not be recognized.

That string is `UNRECOGNIZED_CONTENT_MESSAGE` (spec §18).

## Formats

| Format     | Extensions                         | Source tag on `Song` |
| ---------- | ---------------------------------- | -------------------- |
| ChordPro   | `.cho`, `.crd`, `.chopro`, `.chordpro`, `.pro` | `chordPro`           |
| Plain text | `.txt`, `.text`                    | `plainText`          |

Original input is stored on `SongSource.original_content`. Key changes must not destroy it.

## ChordPro

Inline chords:

```text
[C]Hello [G]world
```

Supported metadata / structure (including common aliases such as `{t}`, `{st}`, `{k}`, `{soc}` / `{eoc}`):

- `{title}`, `{artist}` / `{composer}` / `{subtitle}`, `{album}`
- `{key}`, `{tempo}` / `{bpm}`, `{time}`
- `{capo}` → song notes (`Capo: N`), not a domain capo field
- `{comment}` and `#` lines → annotation tokens
- `{start_of_verse}` / `chorus` / `bridge` / `prechorus` / `intro` / `outro` / `solo` / `instrumental` / `tab` and matching `end_of_*`
- `{new_song}` stops further songs in the same file and emits a warning
- `{define}` / `{chord}` ignored (no fret diagrams yet)

Unclosed `{…` or `[…` keep the usable portion and warn. Unknown directives warn. Unknown chord symbols are preserved as unrecognized tokens.

Default section when none is declared: Verse.

## Plain text

Chord-over-lyrics uses monospaced columns:

```text
C          G
Amazing grace how sweet
```

Each chord token stores `column` (and `lyric_index` equal to that column) so the viewer can align later.

Also accepted:

- `Title:` / `Artist:` / `Album:` / `Key:` / `Tempo:` / `Time:` / `Capo:` prefixes
- Section headers: `Verse 1`, `[Chorus]`, `Pre-Chorus`, …
- Lyric-only lines (kept, not discarded)
- Chord-only lines (tokens with column positions)
- `N.C.` / `NC` / `%` as annotations, not chords

A line is a chord line only when a **majority** of whitespace-separated tokens are **fully** recognized chords (or N.C.). Partial tokens such as `Amazing` do not count, so lyric lines are not misclassified.

`[Chorus]` is a section header, not ChordPro.

## Detection

`detect_format` treats input as ChordPro when any line starts with `{` or contains an inline `[Chord]` immediately followed by non-whitespace (not a section name like `[Chorus]`). Otherwise it is plain text.

## Warnings

| Kind                     | Typical cause                          |
| ------------------------ | -------------------------------------- |
| `UnrecognizedChord`      | Symbol kept as written                 |
| `PartialChord`           | Parser accepted a prefix only          |
| `UnrecognizedDirective`  | Unknown `{…}`                          |
| `MalformedInput`         | Unclosed brackets, bad key/tempo, empty song |
| `AmbiguousLayout`        | Reserved for later heuristics          |
| `SkippedContent`         | Extra songs after `{new_song}`         |

## Out of scope

- Import UI / file picker / paste field (Phase 5)
- Web URL import
- MusicXML / MXL
- Durable library save (Phase 6)
- Export to ChordPro
