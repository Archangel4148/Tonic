# State management

Application state has a single ownership model. Phase 6 owns the song library in Rust and presentation prefs in the UI.

## Authoritative state

Owned by Rust application services (`tonic-app`), not by React and not by the persistence crate.

Phase 6 authoritative state owned by `AppServices`:

- Application identity (`AppInfo`)
- Persistence health, derived from the `SongLibrary` boundary
- In-memory song library (`StoredSong`: domain `Song` + favorite/tags/recents)
- Current session (open song id + import warnings + transpose steps)

Domain authoritative song data (Phase 3, in `tonic-domain`):

- `Song` documents: written chord tokens, original/performance keys, source text
- Display chords after a key change are **derived** (`Song::display_chord`)

Later authoritative application state will include:

- Setlists and setlist-entry overrides
- User display and transposition preferences stored in Rust

Persistence stores a durable copy of authoritative documents. It does not become the live source of truth while the app is running. On startup, services will load from persistence into memory; during a session, memory wins until a successful save.

## Derived state

Recalculated from authoritative state. Do not store it as a second source of truth.

Derived (recalculated, not stored as truth):

- Transposed/display chords (`Song::display_chord` → `SongSessionView`)
- Rendered chord/lyric layout in React

Presentation state in the UI:

- Loading / error / ready status
- Import textarea draft
- Theme and type scale (`localStorage`)

That UI status is not domain data. It only describes whether IPC succeeded.

## UI rules

- React may keep local widget state (scroll position, open dialogs, form drafts before save).
- React must not parse chords, transpose, or keep a parallel song document.
- After a successful save/load path exists, the UI reads songs through application services (via Tauri commands), not by mutating imported source text directly.

## Persistence rules

- Phase 6: filesystem JSON round-trips songs across restarts. Write-through after import, transpose, metadata, favorite, duplicate, and delete.
- Persistence is a durable snapshot, not live truth.
- Setlist entries will reference song IDs. They must not embed a full copy of the song document.

## Diagram

```text
┌──────────────────────────────────────────────┐
│ React                                        │
│  presentation state only                     │
│  (loading, layout, unsaved editor drafts)    │
└──────────────────────┬───────────────────────┘
                       │ invoke (app_info, import, library, transpose, …)
                       ▼
┌──────────────────────────────────────────────┐
│ Tauri shell                                  │
│  IPC adapter, no song ownership              │
└──────────────────────┬───────────────────────┘
                       ▼
┌──────────────────────────────────────────────┐
│ tonic-app :: AppServices                     │
│  authoritative in-memory library + session   │
└────────┬──────────────┬──────────────┬───────┘
         ▼              ▼              ▼
   tonic-domain   tonic-import   tonic-persist
   (pure logic)   (parsers)      (durable copy)
```
