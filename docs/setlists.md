# Setlists (Phase 8)

Setlists are ordered rehearsal/performance lists. They reference library song ids; they never copy song documents. The same song can appear more than once, and each slot keeps its own key, capo, and notes.

## Model

`StoredSetlist` (`tonic-persist`):

- `id` — `setlist-{n}`
- `name`, optional `notes`, optional `eventDate`
- `entries` — ordered `SetlistEntry` records
- `updatedAt`

Each `SetlistEntry`:

- `id` — `entry-{n}` (global, so the same song can appear twice)
- `songId` — library song reference only
- optional `performanceKey`, `capoFret` (`0..=12`), `notes`

On disk:

```text
{app_data}/library/
├── index.json          { "nextId", "nextSetlistId", "nextEntryId" }
├── songs/{id}.json
└── setlists/{id}.json
```

## Session behavior

Opening a setlist entry clones the library `Song` for display only. Overrides apply on that view:

- Entry performance key → display chords / “Now” key
- Entry capo → `playedKey` via domain `played_key` (not a `Song` field)
- Library song document is unchanged

Transpose / key / reset while a setlist entry is open persist to **that entry**, not the song. Opening the same song from the library still uses the song’s own keys.

Missing songs stay as entries and show `(missing song)`.

## IPC

| Command                 | Purpose                                      |
| ----------------------- | -------------------------------------------- |
| `setlist_list`          | Summaries, sorted by name                    |
| `setlist_get`           | Full setlist with resolved entry titles      |
| `setlist_create`        | New empty setlist (`Untitled setlist`)       |
| `setlist_update_meta`   | Name / notes / event date                    |
| `setlist_delete`        | Remove setlist; songs stay in the library    |
| `setlist_duplicate`     | Copy with new setlist + entry ids            |
| `setlist_add_song`      | Append a song reference                      |
| `setlist_remove_entry`  | Drop one slot                                |
| `setlist_move_entry`    | Reorder by index                             |
| `setlist_update_entry`  | Per-entry key, capo, notes                   |
| `setlist_open_entry`    | Open viewer with setlist context             |

`SongSessionView.setlist` is `SetlistContextView` when opened from a setlist (name, index/total, capo, entry notes, played key).

## UI

Sidebar **Songs | Setlists**. Setlist panel: rename, event/date, notes, add from library, reorder, per-entry key/capo/notes, duplicate, delete. Viewer banner shows setlist name, position, capo, and played key.

React still does not parse or transpose. Capo math lives in `tonic-domain`.

## Out of scope

- Live / performance mode (fullscreen, next/previous, auto-scroll, keep-awake) — Phase 9
