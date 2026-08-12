# Technology choices

These choices follow the spec's required stack and keep Phase 1 small enough to review.

## Application shell

**Tauri 2** is required. It compiles one Rust + web codebase to desktop installers and Android APKs.

- Product name: `Tonic`
- Bundle identifier: `com.tonic.app`
- Desktop is the Phase 1 launch target
- Android packaging is deferred until later phases (`tauri android init` has not been run)

## Backend / domain

**Rust** owns music theory, parsing, file I/O, and local persistence.

Phase 1 splits that work into workspace crates so later phases do not have to invent boundaries:

| Crate                 | Responsibility                                                       |
| --------------------- | -------------------------------------------------------------------- |
| `tonic-domain`        | Music theory and canonical song model. Serde JSON only; no UI/Tauri. |
| `tonic-app`           | Application services and in-memory authoritative state.              |
| `tonic-persist`       | Local JSON song library (filesystem + in-memory test double).        |
| `tonic-import`        | ChordPro and plain-text import into `Song`. No UI/Tauri.             |
| `tonic` (`src-tauri`) | Windowing, IPC, and OS integration.                                  |

The popular gRPC crate also named `tonic` is not a dependency. Our crates use the `tonic-*` prefix to stay distinct.

## Frontend

**React 19 + TypeScript + Vite** is the UI stack.

React was chosen because it is the product owner's most familiar UI framework and works well with Tauri's webview. TypeScript is strict. Vite is the Tauri-supported frontend bundler.

Phase 1 does not add a UI state library, router, or CSS framework. The shell is plain React and CSS.

## Persistence

Phase 6 uses **filesystem JSON** under the Tauri app data directory, behind a `SongLibrary` trait (`FileLibrary` / `MemoryLibrary`). SQLite was not required to meet “songs survive restarts offline.” See [`persist.md`](./persist.md).

## Testing and quality

| Tool                       | Role                                          |
| -------------------------- | --------------------------------------------- |
| `cargo test`               | Domain, app, persist, and import unit tests   |
| `cargo clippy`             | Rust lints, warnings denied in `npm run lint` |
| `rustfmt`                  | Rust formatting                               |
| Vitest + Testing Library   | React shell tests                             |
| ESLint + typescript-eslint | TypeScript/React linting                      |
| Prettier                   | Frontend formatting                           |

## Dependencies kept from the Tauri template

`tauri-plugin-opener` ships with the Tauri 2 React template. Phase 1 does not use it. It is retained as a small, official plugin rather than immediately diverging from the template. It can be removed if it stays unused after URL import and desktop integration work.

No other third-party music, database, or UI libraries were added.
