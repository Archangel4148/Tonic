# Phase 8 — Setlists

**Status:** Complete  
**Phase 9 may proceed.**

## Goal

Support organized rehearsal and performance via setlists.

## What shipped

- `StoredSetlist` / `SetlistEntry` in `tonic-persist` (JSON under `library/setlists/`)
- `AppServices` setlist CRUD, reorder, duplicate, per-entry key/capo/notes
- Opening an entry applies overrides on a session clone; the library `Song` is not mutated
- Transpose / key / reset in setlist context persist to the **entry**
- Tauri IPC + React: Songs | Setlists sidebar, setlist panel, viewer banner + capo
- Docs: [`../setlists.md`](../setlists.md)
- Product phase reported by `AppServices` is **8**

No live/performance mode (fullscreen, next/previous navigation, auto-scroll, keep-awake).

## Acceptance criteria

| Criterion | Result |
| --------- | ------ |
| Same song multiple times without duplicating the song document | Distinct `entry-{n}` ids, shared `songId`; duplicate setlist mints new entry ids |
| Independent performance settings per entry | Key, capo, and notes stored on the entry; viewer uses a display clone |

## Review notes

- UI still never parses or transposes locally.
- Setlists reference song ids only.
- Persistence remains a snapshot; `AppServices` is live truth.
- Capo is setlist/display context, not a domain `Song` field. `played_key` is derived when capo and performance key are both set.

## Known limitations

- No live mode (Phase 9)
- Unsaved editor drafts still do not survive restart
- ChordPro import polish still deferred

## How to review

```bash
npm run tauri dev
npm test
```

In the app: import or open two songs (or the same song twice). **Setlists → New setlist**, add the song twice, set different keys/capos, open each slot. Confirm the library song’s key is unchanged. Duplicate the setlist; delete it; songs remain.
