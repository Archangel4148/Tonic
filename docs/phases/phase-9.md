# Phase 9 — Live Performance Mode

**Status:** Complete  
**Phase 10 may proceed.**

## Goal

Make the application practical for real performances.

## What shipped

- Live / performance view: fullscreen, dark stage theme, large type
- Setlist previous/next (`open_setlist_neighbor`, skips missing songs)
- Setlist progress bar
- Auto-scroll with speed control, pause, and reset to top
- Instant transpose in live chrome (Rust engine)
- Keep-awake via Screen Wake Lock when supported
- Swipe + desktop keyboard navigation
- Optional hide-info toggle
- Docs: [`../live-mode.md`](../live-mode.md)
- Product phase reported by `AppServices` is **9**

No MusicXML, web URL import, or Android project generation.

## Acceptance criteria

| Criterion                                             | Result                                                                       |
| ----------------------------------------------------- | ---------------------------------------------------------------------------- |
| Perform an entire set without returning to the editor | Open setlist → Live → next/prev, transpose, auto-scroll, exit only when done |

## Review notes

- UI still never parses or transposes locally.
- Live mode does not own songs; it only presents `SongSessionView` and calls IPC.
- Forced dark + live type scale are not written over the user’s normal theme/editor sizes.
- Fullscreen uses the Tauri window API when running in the desktop app.

## Known limitations

- Wake lock depends on the WebView; unsupported platforms simply stay usable without it
- No wrap-around at the ends of a setlist
- ChordPro import polish still deferred
- MusicXML is Phase 10

## How to review

```bash
npm run tauri dev
npm test
```

In the app: open a setlist with at least two songs → **Perform set** (or open a song → **Live**). Confirm fullscreen/dark/large type, next/prev (or swipe / arrow keys), auto-scroll + speed, transpose, `Escape` back to the editor/library. Play a single song in Live without a setlist; previous/next stay disabled.
