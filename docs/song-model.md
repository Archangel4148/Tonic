# Canonical song model

The in-memory song document lives in `tonic-domain`. Import, editor, renderer, transposition display, library, and setlists must all consume this model. They must not reparse raw source text.

Phase 3 defines the types and JSON round-trip. ChordPro import is Phase 4. Durable storage is Phase 6.

## Document shape

```text
Song
 ├── id, title, artist, album
 ├── original_key, performance_key
 ├── tempo, time_signature
 ├── notes, created_at, updated_at
 ├── source (format + originalContent + optional url/website)
 └── sections[]
      └── lines[]
           └── tokens[]: Chord | Lyric | Annotation
```

## Authoritative vs derived

| Authoritative             | Derived                           |
| ------------------------- | --------------------------------- |
| Written chord tokens      | Display chords after a key change |
| `original_key`            | Semitone offset between keys      |
| `performance_key`         |                                   |
| `source.original_content` |                                   |

Changing performance key **does not** rewrite chord tokens or destroy import source. Call `Song::display_chord` (which uses the Phase 2 engine) for the spelled chord to show.

If either key is missing, display uses the written chord as-is.

## Positions

Each line is an ordered token list.

- **Inline (ChordPro-style):** `Chord`, `Lyric`, `Chord`, `Lyric`, … Lyric index can be omitted; `Line::chord_lyric_alignments()` infers it from preceding lyric length.
- **Chord-over-lyric:** one lyric token plus chord tokens with `column` and/or `lyric_index`.

`lyric_index` is a Unicode scalar count into the line’s concatenated lyric text. `column` is the monospaced visual column when that layout was imported.

## Source metadata

`SongSource` preserves the import when practical:

- `chordPro` / `plainText` / `web` / `manual` / `other`
- `originalContent` optional
- `url` / `website` for web imports

Normalization and key changes must leave this intact.

## Serialization

JSON via serde (`Song::to_json` / `Song::from_json`). Notes and keys serialize as symbols (`"F#"`, `"Am"`). Chords serialize as structured components, not a single opaque string.

This JSON is the interchange form for tests and later persistence. It is not itself the Phase 6 storage engine.

## Identifiers

`SongId` is an opaque string. The domain does not generate UUIDs; the application/persistence layer will assign stable IDs.

## Out of scope here

- ChordPro / plain-text parsers (Phase 4)
- Viewer / transpose UI (Phase 5)
- Saving to disk (Phase 6)
- Setlists (Phase 8)
