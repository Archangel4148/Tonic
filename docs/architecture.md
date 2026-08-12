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
   ┌────┴────┐
   ↓         ↓
Domain    Persistence
(tonic-    (tonic-
 domain)   persist)
```

The spec diagram lists domain above persistence. That is treated as a **layering** diagram, not as "domain depends on persistence."

Domain code must stay independently unit-testable. Persistence will depend on domain types once songs exist. Application services orchestrate both.

## Invariants

1. UI components must not contain music-theory algorithms.
2. Chord parsing and transposition must not depend on UI components.
3. Persistence is not the authoritative representation of active application state.
4. There is one canonical in-memory song representation (introduced in Phase 3).
5. The renderer consumes the canonical model rather than reparsing raw text.
6. Setlists reference songs; they do not copy song documents.
7. There is one transposition implementation, living in `tonic-domain`.
8. The domain crate can be tested with `cargo test -p tonic-domain` and has no Tauri/React dependencies.

## Current crate roles

### `tonic-domain`

Pure domain logic. Phase 2 implements notes, keys, chord parsing, transposition, and capo math. See [`music-theory.md`](./music-theory.md). The canonical song model arrives in Phase 3.

### `tonic-app`

Owns running-process authoritative state. Phase 1 holds `AppServices`, which reports identity and persistence health. Song library, setlists, and preferences will be added here rather than in React or the Tauri crate.

### `tonic-persist`

Persistence interface. Phase 1 provides `Store` and `MemoryStore`. SQLite or filesystem storage is Phase 6. Import/export parsers are not implemented yet; Phase 4 will decide whether they live here or in a dedicated import crate.

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

- Canonical song/setlist types (Phase 3)
- Import/export (Phase 4+)
- Transpose UI / viewer (Phase 5)
- Durable storage (Phase 6)
- Library, editor, or performance UI
- Android project generation
- Cloud, accounts, or telemetry
