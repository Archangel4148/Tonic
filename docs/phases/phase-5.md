# Phase 5 — Song Viewer

**Status:** Complete  
**Phase 6 may proceed.**

## Goal

Create the first genuinely usable song-reading experience.

## What shipped

- Import UI: paste, open file, sample charts (ChordPro / plain text / unknown chords)
- Song viewer: section headers, chord/lyric alignment, wrap-friendly syllables
- Transpose: ± semitone, key dropdown, reset (Rust engine only)
- Independent lyric / chord / header size controls
- Dark / light / system themes (dark default)
- Engine status available but not dominating the chrome
- `SongSessionView` DTO from `tonic-app`; IPC in the Tauri shell
- Docs: [`../viewer.md`](../viewer.md)

No library persistence, editor, setlists, live mode, web import, or MusicXML.

## Acceptance criteria

| Criterion                                       | Result                                                                |
| ----------------------------------------------- | --------------------------------------------------------------------- |
| Import a song                                   | Paste/file/sample → `import_song` → viewer                            |
| View it                                         | Sections, lyrics, display chords                                      |
| Change its key                                  | `−` / `+` / dropdown; written chords unchanged                        |
| Read chords and lyrics without losing alignment | Syllable columns from `lyricIndex`; wrap keeps chord with lyric slice |

## Review notes

- UI never reparses chart text or transposes locally.
- Current song is in-memory only; quitting the app discards it.
- Product phase reported by `AppServices` is **5**.

## Known limitations

- No durable save (Phase 6)
- No editor (Phase 7)
- Capo is still notes text, not a transpose control
- Font/theme prefs are `localStorage` only
- `{new_song}` still skips extra songs in one file

## How to review

```bash
npm run tauri dev
npm test
```

In the app: load **Amazing Grace (ChordPro)**, confirm G/D alignment, press **+** and check chords move while lyrics stay put, toggle **Light**, bump lyric size.
