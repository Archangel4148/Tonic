# Testing

Testing is mandatory alongside implementation. Do not delete or weaken tests to make a feature pass.

## Commands

| Command                      | What it runs                            |
| ---------------------------- | --------------------------------------- |
| `npm test`                   | UI tests, then the full Rust workspace  |
| `npm run test:ui`            | Vitest once                             |
| `npm run test:ui:watch`      | Vitest watch mode                       |
| `npm run test:rust`          | `cargo test --workspace`                |
| `cargo test -p tonic-domain` | Domain crate only, no UI                |
| `npm run lint`               | ESLint + Clippy (`-D warnings`)         |
| `npm run format:check`       | Prettier + rustfmt check                |
| `npm run check`              | Format, lint, tests, and frontend build |

## Rust

Each workspace crate has unit tests next to the code.

Phase 1 coverage:

- `tonic-domain` can run without UI/Tauri dependencies
- `tonic-persist` error messages are preserved
- `tonic-app` wires domain + persist and reports application identity

Phase 2 coverage (`cargo test -p tonic-domain`):

- Note parsing, enharmonics, invalid notes
- Key parsing, enharmonic keys, diatonic spelling (`Ab` vs `G#`)
- Chord parse: required families, slash chords, partial and unrecognized input
- Semitone transpose acceptance examples (`C→D`, `F#m7b5/C#→G#m7b5/D#`, …)
- Key-to-key transpose spelling
- Capo sounding vs played shapes

Phase 3 coverage:

- Song without raw source text
- Inline and chord-over-lyric position recovery
- JSON round-trip equality
- Performance-key display does not mutate written chords or source

Phase 5 coverage (`cargo test -p tonic-app` + UI):

- Import stores a session and returns display chords
- Transpose changes performance key, not written chords or source
- Missing original key is inferred from the first chord
- Viewer/import/transpose UI against mocked IPC

Phase 4 coverage (`cargo test -p tonic-import`):

- ChordPro metadata, sections, and inline chord positions (`amazing_grace.cho`)
- Unknown ChordPro chords preserved; slash chords fully recognized
- Malformed ChordPro keeps title, lyrics, and usable chords
- Plain-text chord-over-lyrics column alignment
- Lyric-only lines and mixed charts
- Unknown plain-text chords preserved
- Content-based format detection (ChordPro vs `[Chorus]` plain text)

## Frontend

Vitest uses jsdom and Testing Library.

Tauri `invoke` is mocked in `src/test/setup.ts` so UI tests do not require a running webview. Tests cover:

- Shell render after a successful `app_info` response
- Import → viewer (title, section, lyrics, chords)
- Transpose IPC (lyrics unchanged, display chords update)
- Visible error when the engine is unavailable
- Chart-line syllable splitting
- Viewer warnings and unrecognized chords

Phase 7 coverage:

- Domain line tag/untag and lyric clamp
- Create → tag chords → save → reopen from disk
- Cancel new song does not persist; cancel edit restores saved lyrics
- Replace unrecognized chord; parse-body paste
- UI New song opens editor (mocked IPC)

Phase 6 coverage:

- `tonic-persist` file library round-trip + delete
- `tonic-app` import persists; reopen after `AppServices::open`
- Search, favorite, tags, duplicate, delete
- UI library list + open against mocked IPC

## What is not tested yet

- End-to-end Tauri window launch (manual: `npm run tauri dev`)
- Android
- End-to-end transpose in a real Tauri window (manual: `npm run tauri dev`)

Those arrive with the phases that implement them.

## Regression rule

When a bug is found, add a deterministic regression test when practical.
