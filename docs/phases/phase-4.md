# Phase 4 — ChordPro & Plain Text Import

**Status:** Complete  
**Phase 5 may proceed.**

## Goal

Allow users to turn real-world chord sheets into canonical songs.

## What shipped

- Dedicated `tonic-import` crate (depends on `tonic-domain` only)
- ChordPro parser (`.cho` / `.crd`, inline `[C]Hello [G]world`, metadata + section directives)
- Plain-text chord-over-lyrics parser (column alignment)
- Format detection from content and file extension
- Non-fatal warnings; `UNRECOGNIZED_CONTENT_MESSAGE`
- Unknown / partial chords preserved on the `Song` model
- Fixtures + integration tests under `crates/import/`
- `AppServices::import_song` orchestration (no IPC / UI yet)
- Docs: [`../import.md`](../import.md)

No viewer, transpose UI, web URL import, MusicXML, or durable storage.

## Acceptance criteria

| Criterion                                         | Result                                                                      |
| ------------------------------------------------- | --------------------------------------------------------------------------- |
| Imported content is editable/renderable via model | Parsers emit `Song` sections, lines, and tokens                             |
| Malformed input does not destroy usable content   | Unclosed `{`/`[`, unknown directives, and unknown chords still yield a song |

## Review notes

- Import is not persistence. `tonic-persist` stays an in-memory stub.
- Written chords remain authoritative; display transposition is still derived (`Song::display_chord`).
- Capo in ChordPro/plain metadata is stored as song notes, not a domain capo field.
- Product phase reported by `AppServices` is **4**.

## Known limitations

- No import IPC or UI (Phase 5)
- No song viewer or transpose controls (Phase 5)
- No library persistence (Phase 6)
- No web URL import
- No MusicXML (Phase 10)
- `{new_song}` skips remaining songs instead of splitting into multiple documents
- Chord diagrams (`{define}`) are ignored

## How to review

```bash
cargo test -p tonic-import
cargo test --workspace
npm test
```
