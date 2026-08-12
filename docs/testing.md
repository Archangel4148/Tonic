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
- `tonic-persist` memory stub is healthy and preserves error messages
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

## Frontend

Vitest uses jsdom and Testing Library.

Tauri `invoke` is mocked in `src/test/setup.ts` so UI tests do not require a running webview. Tests cover:

- Shell render after a successful `app_info` response
- Visible error when the engine is unavailable

## What is not tested yet

- End-to-end Tauri window launch (manual: `npm run tauri dev`)
- Android
- Import, song rendering, durable persistence round-trips
- UI-level transposition (engine is tested in Rust only)

Those arrive with the phases that implement them.

## Regression rule

When a bug is found, add a deterministic regression test when practical.
