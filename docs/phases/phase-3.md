# Phase 3 — Canonical Song Model

**Status:** Complete  
**Phase 4 may proceed.**

## Goal

Build the normalized representation used throughout the application.

## What shipped

- `Song`, metadata, source, sections, lines, chord/lyric/annotation tokens
- Recoverable chord-to-lyric positions (inline inference + explicit column/index)
- JSON serialize/deserialize (`Song::to_json` / `from_json`)
- Display transposition derived from original → performance key (does not mutate written chords or source)
- Docs: [`../song-model.md`](../song-model.md)

No import parsers, viewer, or persistence were added.

## Acceptance criteria

| Criterion                             | Result                                                 |
| ------------------------------------- | ------------------------------------------------------ |
| Song usable without raw source text   | Manual song with tokens only, `originalContent` absent |
| Chord and lyric positions recoverable | `Line::chord_lyric_alignments()` + JSON round-trip     |

## Review notes

- One canonical in-memory song type, in `tonic-domain`
- `tonic-domain` now depends on `serde` and `serde_json` for the model interchange format
- Still no Tauri/React dependency on the domain crate

## Known limitations

- No ChordPro/plain-text import yet (Phase 4)
- No song viewer (Phase 5)
- JSON is not durable library storage (Phase 6)
- `SongId` is not auto-generated
- MusicXML/score representation is still Phase 10

## How to review

```bash
cargo test -p tonic-domain
cargo test --workspace
npm test
```
