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
│   └── persist/               tonic-persist
├── src/                       React + TypeScript UI
│   ├── App.tsx                Application shell
│   ├── lib/                   IPC wrappers and shared UI types
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
| `com.tonic.app` | Bundle identifier                                                     |

## Frontend layout rule

Keep `src/` flat until a later phase needs more structure. Do not add routers, stores, or feature folders ahead of the UI that would use them.

## Generated / ignored output

- `target/` — Rust build artifacts (workspace root)
- `dist/` — Vite production build
- `src-tauri/gen/` — Tauri-generated capability schemas and later mobile projects
- `node_modules/` — npm dependencies
