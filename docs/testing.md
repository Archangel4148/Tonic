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
- `tonic-app` wires domain + persist and reports Tonic Phase 1 identity

Music-theory tests in later phases should be exhaustive and explicit: valid chords, invalid input, enharmonics, slash chords, and key context.

## Frontend

Vitest uses jsdom and Testing Library.

Tauri `invoke` is mocked in `src/test/setup.ts` so UI tests do not require a running webview. Tests cover:

- Shell render after a successful `app_info` response
- Visible error when the engine is unavailable

## What is not tested yet

- End-to-end Tauri window launch (manual: `npm run tauri dev`)
- Android
- Import, transposition, rendering, persistence round-trips

Those arrive with the phases that implement them.

## Regression rule

When a bug is found, add a deterministic regression test when practical.
