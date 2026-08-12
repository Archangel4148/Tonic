# Phase 1 — Foundation & Architecture

**Status:** Implemented, pending review  
**Do not start Phase 2 until explicitly instructed.**

## Goal

Create the project skeleton and establish architectural boundaries without implementing later-phase features.

## What shipped

- Tauri 2 + React 19 + TypeScript + Vite application named **Tonic**
- Rust workspace with `tonic-domain`, `tonic-app`, `tonic-persist`, and the Tauri shell
- Basic application shell that loads engine status over IPC
- In-memory persistence stub behind a `Store` trait
- Authoritative state owned by `AppServices` in Rust
- Vitest + Testing Library for the UI
- `cargo test` / Clippy / rustfmt for Rust
- ESLint + Prettier for the frontend
- Documentation under `docs/`

## Acceptance criteria

| Criterion                      | Result                                                                 |
| ------------------------------ | ---------------------------------------------------------------------- |
| Application launches           | `npm run tauri dev` compiled and started `tonic.exe`                   |
| Test suite runs                | Rust: 4 passed. UI: 2 passed                                           |
| Build succeeds                 | Frontend production build + Cargo workspace build                      |
| Architecture is documented     | `docs/architecture.md` and related docs                                |
| Domain code runs without UI    | `cargo test -p tonic-domain` — crate has zero dependencies             |
| No unnecessary future features | No chord engine, song model, library, editor, or MusicXML              |

## Review notes

See the checkpoint report in the Phase 1 implementation summary:

- Technology choices: [`../technology-choices.md`](../technology-choices.md)
- Project structure: [`../project-structure.md`](../project-structure.md)
- Dependency graph: [`../architecture.md`](../architecture.md)
- State ownership: [`../state-management.md`](../state-management.md)
- Test setup: [`../testing.md`](../testing.md)
- Known limitations: below

## Known limitations

- No music-theory engine yet (Phase 2)
- No canonical song model yet (Phase 3)
- No import/export (Phase 4+)
- Persistence does not survive restarts
- `npm run dev` in a browser cannot call Rust IPC
- Android project is not initialized
- App icons are still the Tauri template icons
- `tauri-plugin-opener` is unused
- No CI workflow yet
- No custom license/branding beyond the name Tonic

## How to review

1. Read `docs/architecture.md` and `docs/state-management.md`
2. Run `npm install`
3. Run `npm test`
4. Run `npm run lint`
5. Run `npm run tauri dev` and confirm the shell shows Phase 1 engine status

The product spec now lives at `docs/spec.md`.
