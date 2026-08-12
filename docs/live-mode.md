# Live performance mode (Phase 9)

A stage-first reading view: large type, dark theme, setlist next/previous, instant transpose, auto-scroll, and keep-awake where the platform allows it. The musician can run a whole set without returning to the editor.

Live mode is **presentation state**. Song documents, setlist entries, and transpose still live in `AppServices`. React only scrolls, layouts, and invokes IPC.

## Enter / exit

- **Live** on the song toolbar, or **Perform set** on a setlist with at least one playable song (opens the current or first entry).
- **Exit live**, or `Escape`. Fullscreen and wake lock are released. Theme and editor type scale are restored.

## Stage presentation

- Near-fullscreen window (`@tauri-apps/api/window`, Fullscreen API fallback)
- Forced dark theme for the session (not persisted over the user’s theme preference)
- Larger live type scale (stored separately in `localStorage` as `tonic-live-type-scale`)
- Thin setlist progress bar; chrome auto-hides after a few seconds of idle. Move the pointer, tap, or press a key to show controls again.
- **Lock** (or `L`) hides all controls except a small Unlock button. Hotkeys still work.

## Navigation

| Input                             | Action                                                    |
| --------------------------------- | --------------------------------------------------------- |
| Previous / Next, swipe left/right | Adjacent playable setlist entry (`setlist_open_neighbor`) |
| Pinch in / out                    | Scale lyrics, chords, section labels, and sheet music     |
| `Ctrl`+scroll / `Ctrl`+`−`/`+`    | Same text scale (also `Cmd` on macOS)                     |
| `←` / `PageUp`, `→` / `PageDown`  | Previous / next                                           |
| `Space`                           | Start/stop auto-scroll                                    |
| `Home`                            | Scroll to top                                             |
| `−` / `+`                         | Transpose one semitone (Rust)                             |
| `[` / `]`                         | Slower / faster auto-scroll                               |
| `M`                               | Toggle extra metadata                                     |
| `L`                               | Lock / unlock on-screen controls                          |
| `F11` / `Alt+Enter`               | Toggle fullscreen                                         |
| `Escape`                          | Exit live                                                 |

Chrome **−** / **+** text-size buttons bump the same live scale as pinch (persisted separately from the editor scale).

Missing setlist songs are skipped. At either end, navigation stops (no wrap). A single song (no setlist) can still use live mode; previous/next stay disabled.

## Auto-scroll

- Start/stop, speed slider (`8`–`90` px/s, persisted), reset to top
- Scroll position is kept as a fraction so speeds below 1px/frame still move, and faster speeds stay distinct
- Stops at the bottom. Changing song resets to the top and pauses.

## Keep-awake

Uses the Screen Wake Lock API when present (WebView2 / Chromium). If unavailable, live mode still works.

## IPC

| Command                 | Purpose                                        |
| ----------------------- | ---------------------------------------------- |
| `setlist_open_neighbor` | `delta` +1 / −1 from the current setlist entry |

Transpose, key, and reset use the existing session commands (entry overrides if a setlist is open).

Sheet music imported in Phase 10 renders in live mode through the same `SongViewer` / `sheetMusicXml` path.

## Out of scope

- Android keep-awake beyond the Screen Wake Lock API when the WebView exposes it
