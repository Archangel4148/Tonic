# MusicXML & sheet music (Phase 10)

Tonic imports MusicXML / compressed MXL into a canonical **score** model, then renders derived MusicXML with OpenSheetMusicDisplay. Scores are not forced into the chord-chart token list (spec §10.3).

## Ownership

| Layer          | Role                                                                                          |
| -------------- | --------------------------------------------------------------------------------------------- |
| `tonic-domain` | `Score`, measures, notes, rests, harmony; `Score::transpose_semitones` / `to_musicxml`        |
| `tonic-import` | Parse `.musicxml` / `.xml` / `.mxl` → `Song.score` + optional lyric/harmony companion section |
| `tonic-app`    | `import_text` / `import_bytes`; session DTO includes `sheetMusicXml`                          |
| React          | OSMD engraving only. No parsing or transpose in the UI                                        |

Written pitches on `Score` are authoritative for theory/search. **Sheet rendering uses the original MusicXML** (staves, both clefs, beams, dynamics, layout). On transpose, Rust rewrites only pitch / key / harmony-root values in that document.

## Import

| Input                     | How                                                 |
| ------------------------- | --------------------------------------------------- |
| `.musicxml` / `.xml` text | `import` / `import_auto` / paste                    |
| `.mxl` zip                | Unzipped in Rust (`import_bytes` / `import_binary`) |

Detection: `<score-partwise` / `<score-timewise`, or extension `.musicxml` / `.mxl` / `.xml`. Timewise scores are converted to partwise with a warning.

Usable notes, rests, lyrics, and `<harmony>` chord symbols are kept. Unsupported features (tuplets, ornaments, figured bass, directions, …) emit `UnsupportedFeature` warnings and still render the supported portion.

A companion chart section named **Score** is extracted from lyrics + harmony so library search still works. It is not the canonical notation body.

## Rendering

`SongSessionView.sheetMusicXml` is generated in Rust:

1. Prefer `Song.source.originalContent` when the source is MusicXML / MXL
2. If session steps ≠ 0, `transpose_musicxml_text` rewrites pitches, key fifths, and harmony roots only
3. Fall back to `Score::to_musicxml()` only when original XML is missing

The viewer and live mode pass that XML to **OpenSheetMusicDisplay** (SVG backend). OSMD is an engraving library only. Online MusicXML viewers look similar because they also feed OSMD (or equivalent) the full original document.

## Editor

Editing a MusicXML song updates metadata and the extracted chart only. The score is preserved on save. There is no notation authoring.

## IPC

| Command         | Purpose                                               |
| --------------- | ----------------------------------------------------- |
| `import_song`   | Text, including MusicXML (`format: musicXml` or auto) |
| `import_binary` | `bytes: number[]` + optional `fileName` (`.mxl`)      |

## Out of scope

- MusicXML authoring comparable to a notation editor
- Web URL import
- Forcing scores into chart lines as the only representation
