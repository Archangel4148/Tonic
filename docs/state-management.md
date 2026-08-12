# State management

Application state has a single ownership model. Phase 9 owns the song library, setlists, and editor draft in Rust. Live mode, theme, and type scale are presentation state in the UI.

## Authoritative state

Owned by Rust application services (`tonic-app`), not by React and not by the persistence crate.

Phase 9 authoritative state owned by `AppServices`:

- Application identity (`AppInfo`)
- Persistence health, derived from the `SongLibrary` boundary
- In-memory song library (`StoredSong`: domain `Song` + favorite/tags/recents)
- In-memory setlists (`StoredSetlist`: ordered song-id entries + per-entry overrides)
- Current session (open song id + optional setlist/entry id + import warnings + transpose steps)
- Editor draft (`EditorSession`: unsaved `Song` + dirty/new flags)

Domain authoritative song data (Phase 3, in `tonic-domain`):

- `Song` documents: written chord tokens, original/performance keys, source text
- Display chords after a key change are **derived** (`Song::display_chord`)

Later authoritative application state will include:

- User display and transposition preferences stored in Rust

Persistence stores a durable copy of authoritative documents. It does not become the live source of truth while the app is running. On startup, services will load from persistence into memory; during a session, memory wins until a successful save.

## Derived state

Recalculated from authoritative state. Do not store it as a second source of truth.

Derived (recalculated, not stored as truth):

- Transposed/display chords (`Song::display_chord` → `SongSessionView`)
- Setlist played key (`played_key` from performance key + capo)
- Rendered chord/lyric layout in React

Presentation state in the UI:

- Loading / error / ready status
- Import textarea draft
- Editor lyric/metadata form fields before blur/save
- Theme, type scale, and live-mode prefs (`localStorage`)
- Live chrome, auto-scroll playing/speed, scroll position, fullscreen

That UI status is not domain data. It only describes whether IPC succeeded.

## UI rules

- React may keep local widget state (scroll position, open dialogs, form drafts before save).
- React must not parse chords, transpose, or keep a parallel song document.
- After a successful save/load path exists, the UI reads songs through application services (via Tauri commands), not by mutating imported source text directly.

## Persistence rules

- Phase 6+: filesystem JSON round-trips songs (and, from Phase 8, setlists) across restarts. Write-through after import, transpose, metadata, favorite, duplicate, delete, setlist edits, and **editor Save**. Editor Cancel does not write.
- Persistence is a durable snapshot, not live truth.
- Setlist entries reference song IDs. They must not embed a full copy of the song document.

## Diagram

```text
┌──────────────────────────────────────────────┐
│ React                                        │
│  presentation state only                     │
│  (loading, layout, unsaved editor drafts)    │
└──────────────────────┬───────────────────────┘
                       │ invoke (app_info, import, library, setlist, transpose, …)
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
