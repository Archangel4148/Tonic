# Persistence (Phase 6–8)

Tonic stores the song library and setlists as JSON files in a **user-visible folder** so they can be copied to Google Drive or a file manager.

Desktop uses `Documents/Tonic/`. Android prefers `Documents/Tonic` (then `Download/Tonic`) under shared storage. Older installs in the private app-data `library/` folder are copied forward on first launch.

Settings → **Open save folder** reveals that directory. Tonic rereads the folder when you return to the app, or from **Rescan folder**, so Drive copies and deleted files show up without restarting.

## Layout

```text
{Documents}/Tonic/   (or Download/Tonic on some phones)
├── HOW_TO_BACKUP.txt
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

| Layer           | Role                                             |
| --------------- | ------------------------------------------------ |
| `tonic-persist` | Read/write files. Disk is the durable library.                 |
| `tonic-app`     | In-memory cache + open session. Refreshed from disk on rescan. |
| React           | Renders list/session DTOs. Does not parse songs.               |

On startup, `AppServices::open` loads every song and setlist into memory. Edits in the app write through immediately. External folder changes (paste from Drive, delete a JSON file) are picked up by `reload_from_disk`.

## Why filesystem JSON

The spec allowed SQLite, a filesystem store, or IndexedDB. JSON files are the smallest durable option that round-trips the existing serde `Song` model, stays offline, and is easy to inspect during development. SQLite can replace this later without changing the `SongLibrary` trait.

Tests use `MemoryLibrary` (same trait, no disk).

## What is not stored here

- Theme and type scale remain in `localStorage` (presentation only)
- Editor drafts (Phase 7)
