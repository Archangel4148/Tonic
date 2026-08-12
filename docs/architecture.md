# Architecture

Tonic keeps a strict separation between presentation, application services, domain logic, and persistence.

## Dependency direction

```text
UI / Presentation (React)
        ↓  IPC only
    Tauri shell (src-tauri)
        ↓
Application services (tonic-app)
        ↓
   ┌────┴────┬────────┐
   ↓         ↓        ↓
Domain    Persist   Import
(tonic-   (tonic-   (tonic-
 domain)  persist)  import)
```

The spec diagram lists domain above persistence. That is treated as a **layering** diagram, not as "domain depends on persistence."

Domain code must stay independently unit-testable. Persistence will depend on domain types once songs exist. Import depends on domain types only. Application services orchestrate domain, import, and persist.

## Invariants

1. UI components must not contain music-theory algorithms.
2. Chord parsing and transposition must not depend on UI components.
3. Persistence is not the authoritative representation of active application state.
4. There is one canonical in-memory song representation (`Song` in `tonic-domain`).
5. The renderer consumes the canonical model rather than reparsing raw text.
6. Setlists reference songs; they do not copy song documents.
7. There is one transposition implementation, living in `tonic-domain`.
8. The domain crate can be tested with `cargo test -p tonic-domain` and has no Tauri/React dependencies.
9. Import parsers live in `tonic-import`, not in the UI, domain engine, or persistence crate.

## Current crate roles

### `tonic-domain`

Pure domain logic: music engine plus the canonical `Song` model. See [`music-theory.md`](./music-theory.md) and [`song-model.md`](./song-model.md).

### `tonic-app`

Owns running-process authoritative state. Reports identity and persistence health, and orchestrates import via `AppServices::import_song`. Song library, setlists, and preferences will be added here rather than in React or the Tauri crate.

### `tonic-import`

ChordPro and plain-text parsers. Depends on `tonic-domain` only. See [`import.md`](./import.md).

### `tonic-persist`

Persistence interface. Phase 1 provides `Store` and `MemoryStore`. SQLite or filesystem storage is Phase 6. Import parsers live in `tonic-import`, not here.

### `tonic` (`src-tauri`)

Tauri entrypoint. It manages `AppServices` as Tauri state and exposes IPC commands. It must not grow music-theory code.

### React UI (`src/`)

Presentation only. It may hold view state such as loading/error flags. It talks to Rust through `src/lib/tauri.ts` and must not reimplement domain behavior.

## IPC surface (Phase 1)

| Command    | Direction | Purpose                                                                   |
| ---------- | --------- | ------------------------------------------------------------------------- |
| `app_info` | UI → Rust | Return application identity, phase, domain engine, and persistence health |

JSON uses camelCase to match TypeScript.

## What later phases still do not include

- Import IPC / viewer / transpose UI (Phase 5)
- Durable storage (Phase 6)
- Library, editor, or performance UI
- Setlists (Phase 8)
- Web URL import
- MusicXML (Phase 10)
- Android project generation
- Cloud, accounts, or telemetry
