# Phase 6 — Library & Persistence

**Status:** Complete  
**Phase 7 may proceed.**

## Goal

Turn the viewer into a usable songbook: songs survive restarts and stay offline.

## What shipped

- Filesystem JSON library under Tauri app data (`tonic-persist::FileLibrary`)
- In-memory `AppServices` library with write-through save
- Import auto-saves into the library
- Sidebar: search (title/artist/album/tags/lyrics), favorites, artist/key/tag filters, sort, recents
- Metadata editing (title, artist, album, notes, tags) without a chart editor
- Favorite toggle, duplicate, delete
- Transpose / key changes persist on the open song
- Docs: [`../persist.md`](../persist.md)
- Product phase reported by `AppServices` is **6**

No chart editor, new-song composer, setlists, live mode, web import, or MusicXML.

## Acceptance criteria

| Criterion                          | Result                                                          |
| ---------------------------------- | --------------------------------------------------------------- |
| Songs survive application restarts | `AppServices::open` reloads JSON files; reopen test covers this |
| Remain available offline           | Local app-data files only; no network                           |

## Review notes

- UI still never reparses or transposes locally.
- Persistence is not the live source of truth; memory is, with write-through snapshots.
- Library metadata (favorite/tags/recents) is stored beside `Song`, not inside the domain document.
- Closing a song only clears the viewer session; the library entry remains.

## Known limitations

- No chart editor / new song (Phase 7)
- Theme and type scale still `localStorage` only
- One JSON file per song; not SQLite
- ChordPro parsing polish is deferred

## How to review

```bash
npm run tauri dev
npm test
```

In the app: import a chart, quit and relaunch, confirm it is still in the library. Search, favorite, edit details, duplicate, delete, transpose and reopen.
