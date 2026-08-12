# Persistence (Phase 6–8)

Tonic stores the song library and setlists as JSON files under the Tauri app data directory (`…/library/`). Application services keep the live copy in memory and write through on every change.

## Layout

```text
{app_data}/library/
├── index.json          { "nextId", "nextSetlistId", "nextEntryId" }
├── songs/
│   ├── song-1.json
│   └── song-2.json
└── setlists/
    └── setlist-1.json
```

Each song file is a `StoredSong`:

- `song` — canonical `tonic-domain::Song`
- `favorite`, `tags` — library metadata (not domain fields)
- `lastOpenedAt`, `lastModifiedAt` — Unix seconds, used for recents

`SongId` values are `song-{n}`, assigned by `AppServices`. Setlist ids are `setlist-{n}`; entry ids are `entry-{n}` (global so one song can appear twice). Setlist files are `StoredSetlist`: name, notes, event date, and entries that reference `songId` plus optional performance key, capo, and notes. See [`setlists.md`](./setlists.md).

## Ownership

| Layer            | Role                                              |
| ---------------- | ------------------------------------------------- |
| `tonic-persist`  | Read/write files. Not the live source of truth.   |
| `tonic-app`      | In-memory library + open session. Authoritative.  |
| React            | Renders list/session DTOs. Does not parse songs.  |

On startup, `AppServices::open` loads every song and setlist into memory. While running, memory wins until a successful save. Import, transpose, metadata, favorite, duplicate, delete, and setlist edits all persist immediately.

## Why filesystem JSON

The spec allowed SQLite, a filesystem store, or IndexedDB. JSON files are the smallest durable option that round-trips the existing serde `Song` model, stays offline, and is easy to inspect during development. SQLite can replace this later without changing the `SongLibrary` trait.

Tests use `MemoryLibrary` (same trait, no disk).

## What is not stored here

- Theme and type scale remain in `localStorage` (presentation only)
- Editor drafts (Phase 7)
