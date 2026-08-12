# Project structure

```text
Tonic/
├── Cargo.toml                 Workspace root
├── rustfmt.toml
├── package.json               Frontend + Tauri scripts
├── vite.config.ts             Vite + Vitest
├── eslint.config.js
├── docs/                      Project documentation
│   ├── spec.md                Authoritative product spec
│   └── phases/                Per-phase review notes
├── crates/
│   ├── domain/                tonic-domain (music engine + Song model)
│   ├── app/                   tonic-app
│   ├── persist/               tonic-persist
│   └── import/                tonic-import (ChordPro + plain text)
│       ├── src/
│       ├── fixtures/
│       └── tests/
├── src/                       React + TypeScript UI
│   ├── App.tsx                Application shell
│   ├── components/            Import panel, viewer, transpose, theme
│   ├── lib/                   IPC wrappers, types, samples, chart split
│   └── test/                  Test setup
└── src-tauri/                 Tauri shell crate (`tonic` / `tonic_lib`)
    ├── tauri.conf.json
    ├── capabilities/
    ├── icons/
    └── src/
```

## Naming

| Name            | Where it appears                                                      |
| --------------- | --------------------------------------------------------------------- |
| Tonic           | Product / window title / UI                                           |
| `tonic`         | npm package and Tauri binary crate                                    |
| `tonic_lib`     | Tauri library crate name (Windows requires it to differ from the bin) |
| `tonic-domain`  | Domain crate                                                          |
| `tonic-app`     | Application crate                                                     |
| `tonic-persist` | Persistence crate                                                     |
| `tonic-import`  | ChordPro / plain-text import                                          |
| `com.tonic.app` | Bundle identifier                                                     |

## Frontend layout rule

`src/components/` holds viewer, import, library sidebar, and metadata details. Do not add a router, client store, or setlist screens ahead of Phase 8.

## Generated / ignored output

- `target/` — Rust build artifacts (workspace root)
- `dist/` — Vite production build
- `src-tauri/gen/` — Tauri-generated capability schemas and later mobile projects
- `node_modules/` — npm dependencies
