# Phase 7 — Editing & Manual Song Creation

**Status:** Implemented, pending review  
**Do not start Phase 8 until explicitly instructed.**

## Goal

Allow users to create and correct songs without external software.

## What shipped

- New Song → in-memory editor draft (`Untitled`, one Verse)
- Structured song editor: metadata, sections, lyrics, chord tagging, annotations
- Parser correction: retag unrecognized chords; paste ChordPro/plain text to replace the body
- Save / Cancel (draft is not the library until Save)
- Edit existing library songs
- Docs: [`../editor.md`](../editor.md)
- Product phase reported by `AppServices` is **7**

No setlists, live mode, web import, or MusicXML.

## Acceptance criteria

| Criterion | Result |
| --------- | ------ |
| Create a complete chord chart from scratch | New song → lyrics + tagged chords + sections → Save |
| Reopen it later | Library round-trip after Save |

## Review notes

- UI still never parses or transposes locally.
- Editor draft is authoritative while open; disk updates only on Save.
- Transpose and library metadata write-through refuse a dirty editor (“Save or cancel first”).

## Known limitations

- Unsaved drafts do not survive restart
- No setlists (Phase 8)
- ChordPro import polish still deferred

## How to review

```bash
npm run tauri dev
npm test
```

In the app: **New song**, set title/key, type lyrics, tag chords at the caret, add a Chorus, Save, quit, relaunch. Open the song. Edit an imported chart to fix an unrecognized chord, Save.
