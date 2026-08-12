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

Owns running-process authoritative state: identity, persistence health, the in-memory song library, the open session, the editor draft, and import/transpose orchestration. Returns `SongSessionView` / `LibraryListView` / `EditorSessionView` DTOs for the UI. Setlists and durable display preferences will be added here rather than in React or the Tauri crate.

### `tonic-import`

ChordPro and plain-text parsers. Depends on `tonic-domain` only. See [`import.md`](./import.md).

### `tonic-persist`

Local library storage. `FileLibrary` writes JSON under the app data directory; `MemoryLibrary` is for tests. Import parsers live in `tonic-import`, not here. See [`persist.md`](./persist.md).

### `tonic` (`src-tauri`)

Tauri entrypoint. It opens `AppServices` on the app data library path and exposes IPC commands. It must not grow music-theory code.

### React UI (`src/`)

Presentation only. It renders library + `SongSessionView`, holds theme/type-scale prefs, and talks to Rust through `src/lib/tauri.ts`. It must not reimplement domain behavior. See [`viewer.md`](./viewer.md) and [`persist.md`](./persist.md).

## IPC surface (Phase 7)

| Command                     | Direction | Purpose                                        |
| --------------------------- | --------- | ---------------------------------------------- |
| `app_info`                  | UI → Rust | Identity, phase, engine, persistence, key list |
| `import_song`               | UI → Rust | Import chart text into the library + session   |
| `current_song`              | UI → Rust | Current session view, or `null`                |
| `transpose_song`            | UI → Rust | Shift performance key by ±N semitones          |
| `set_performance_key`       | UI → Rust | Set performance key by symbol                  |
| `reset_performance_key`     | UI → Rust | Restore original key                           |
| `clear_song`                | UI → Rust | Close the viewer session (library unchanged)   |
| `library_list`              | UI → Rust | Search/filter/sort library summaries           |
| `library_open`              | UI → Rust | Open a library song                            |
| `library_delete`            | UI → Rust | Delete a library song                          |
| `library_duplicate`         | UI → Rust | Duplicate a song and open the copy             |
| `library_toggle_favorite`   | UI → Rust | Toggle favorite                                |
| `library_update_metadata`   | UI → Rust | Edit title/artist/album/notes/tags             |
| `editor_*`                  | UI → Rust | New/edit draft, save/cancel, sections, tagging |

JSON uses camelCase to match TypeScript. Full editor command list: [`editor.md`](./editor.md).

## What later phases still do not include

- Setlists (Phase 8)
- Web URL import
- MusicXML (Phase 10)
- Android project generation
- Cloud, accounts, or telemetry
