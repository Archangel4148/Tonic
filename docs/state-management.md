# State management

Application state has a single ownership model. Phase 1 only implements enough of that model to prove the boundary.

## Authoritative state

Owned by Rust application services (`tonic-app`), not by React and not by the persistence crate.

Phase 1 authoritative state:

- Application identity (`AppInfo`)
- Persistence health, derived from the `Store` boundary

Later authoritative state will include:

- Song documents
- Library entries
- Setlists and setlist-entry overrides
- User display and transposition preferences

Persistence stores a durable copy of authoritative documents. It does not become the live source of truth while the app is running. On startup, services will load from persistence into memory; during a session, memory wins until a successful save.

## Derived state

Recalculated from authoritative state. Do not store it as a second source of truth.

Examples for later phases:

- Transposed chords
- Search results
- Setlist progress
- Rendered chord/lyric layout

Phase 1 derived/presentation state in the UI:

- Loading / error / ready status for the `app_info` request

That UI status is not domain data. It only describes whether IPC succeeded.

## UI rules

- React may keep local widget state (scroll position, open dialogs, form drafts before save).
- React must not parse chords, transpose, or keep a parallel song document.
- After a successful save/load path exists, the UI reads songs through application services (via Tauri commands), not by mutating imported source text directly.

## Persistence rules

- Phase 1: `MemoryStore` always reports healthy and stores nothing.
- Phase 6: durable storage must round-trip songs and setlists across restarts.
- Setlist entries will reference song IDs. They must not embed a full copy of the song document.

## Diagram

```text
┌──────────────────────────────────────────────┐
│ React                                        │
│  presentation state only                     │
│  (loading, layout, unsaved editor drafts)    │
└──────────────────────┬───────────────────────┘
                       │ invoke("app_info")
                       ▼
┌──────────────────────────────────────────────┐
│ Tauri shell                                  │
│  IPC adapter, no song ownership              │
└──────────────────────┬───────────────────────┘
                       ▼
┌──────────────────────────────────────────────┐
│ tonic-app :: AppServices                     │
│  authoritative in-memory session state       │
└───────────────┬──────────────────┬───────────┘
                ▼                  ▼
        tonic-domain         tonic-persist
        (pure logic)         (durable copy)
```
