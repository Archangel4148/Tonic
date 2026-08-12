# Import

ChordPro, plain-text, and MusicXML parsers live in `tonic-import`. They produce the canonical [`Song`](./song-model.md) model. They do not own music theory, UI, or persistence.

Application code calls `AppServices::import_text` / `import_bytes` (or `tonic_import::import` / `import_auto` / `import_bytes`). Phase 5 exposes text import over IPC (`import_song`); Phase 10 adds `import_binary` for `.mxl`.

## API

```text
import(input, ImportFormat, id) → ImportResult { song, warnings }
import_auto(input, id)          → same, after detect_format(input)
import_bytes(bytes, name, id)   → same, MusicXML/MXL or chart text
format_from_extension(ext)      → Option<ImportFormat>
```

`ImportResult` never hard-fails. Usable chords, lyrics, metadata, and score notes are always kept. If any warning was produced, `summary_message()` is:

> Some content could not be recognized.

except when every warning is `UnsupportedFeature`, in which case it is:

> Some MusicXML features are not supported.

Those strings are `UNRECOGNIZED_CONTENT_MESSAGE` / `UNSUPPORTED_MUSICXML_MESSAGE` (spec §18).

## Formats

| Format     | Extensions                                     | Source tag on `Song` |
| ---------- | ---------------------------------------------- | -------------------- |
| ChordPro   | `.cho`, `.crd`, `.chopro`, `.chordpro`, `.pro` | `chordPro`           |
| Plain text | `.txt`, `.text`                                | `plainText`          |
| MusicXML   | `.musicxml`, `.xml`, `.mxl`                    | `musicXml`           |

Original input is stored on `SongSource.original_content`. Key changes must not destroy it.

## ChordPro

Inline chords:

```text
[C]Hello [G]world
```

Supported metadata / structure (including common aliases such as `{t}`, `{st}`, `{k}`, `{soc}` / `{eoc}`):

- `{title}`, `{artist}` / `{composer}` / `{subtitle}`, `{album}`
- `{key}`, `{tempo}` / `{bpm}`, `{time}`
- `{capo}` / dump markers like `[(capo][+1)]` → inline annotations at that spot (`Capo 1`, `Capo +1`)
- `{comment}` → annotation tokens; `#` comment lines are ignored
- `TIP:` / URL lines → annotations
- `{start_of_verse}` / `chorus` / `bridge` / `prechorus` / `intro` / `outro` / `solo` / `instrumental` / `tab` and matching `end_of_*`
- `{new_song}` / `{ns}` stops further songs only after a song body has started (leading `{ns}` in `.pro` dumps is ignored)
- `[|]`, `[-]`, `[:]`, `[NC]` are layout markers, not chords
- `[INTRO]` / `[Chorus]` lines become section headers
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

`detect_format` treats input as MusicXML when it contains `<score-partwise` / `<score-timewise`, else ChordPro when any line starts with `{` or contains an inline `[Chord]` immediately followed by non-whitespace (not a section name like `[Chorus]`). Otherwise it is plain text.

## Warnings

| Kind                    | Typical cause                                |
| ----------------------- | -------------------------------------------- |
| `UnrecognizedChord`     | Symbol kept as written                       |
| `PartialChord`          | Parser accepted a prefix only                |
| `UnrecognizedDirective` | Unknown `{…}`                                |
| `MalformedInput`        | Unclosed brackets, bad key/tempo, empty song |
| `AmbiguousLayout`       | Reserved for later heuristics                |
| `SkippedContent`        | Extra songs after a mid-file `{new_song}`    |
| `UnsupportedFeature`    | MusicXML construct skipped; notes still kept |

MusicXML / MXL details: [`musicxml.md`](./musicxml.md).

## Web URL import

Paste a supported website URL in Import. Application services fetch the page, then `tonic-import` runs a **site adapter** that extracts chords/lyrics into the canonical `Song` model.

```text
URL → recognize host → fetch HTML → site adapter → Song (+ warnings)
```

| Site | URL shape | Notes |
| ---- | --------- | ----- |
| Ultimate Guitar | `tabs.ultimate-guitar.com/tab/…/…-chords-…` | Reads `js-store` JSON; converts `[ch]Am[/ch]` → ChordPro `[Am]` |

Source metadata: `format: web`, `website: ultimate-guitar`, original URL, and raw chart content when available. After import, the song is local/offline like any other library entry.

Adapters live under `crates/import/src/web/` and are isolated so HTML changes on one site do not touch the domain engine or UI. Unsupported hosts fail with a clear message. Network blocks (bot protection) suggest pasting chart text instead.

IPC: `import_url`. Tests use HTML fixtures (`import_web_html`) so parsing does not require live network access.

## Out of scope

- Additional website adapters (add modularly under `web/`)
- MusicXML authoring
- Export to ChordPro
- Scraping search result catalogs (user-pasted song URLs only)