# Song viewer (Phase 5)

The first usable reading experience: import a chart, view aligned chords and lyrics, and change key. The UI does not parse chords or transpose. Rust owns the song; React renders a display DTO.

## Session ownership

`AppServices` holds the open session song (now also a library entry). Viewer IPC:

| Command                 | Purpose                                     |
| ----------------------- | ------------------------------------------- |
| `app_info`              | Identity, phase, persistence, key list      |
| `import_song`           | Parse text → store song → `SongSessionView` |
| `current_song`          | Session snapshot, or `null`                 |
| `transpose_song`        | ±N semitones on performance key             |
| `set_performance_key`   | Jump to a named key                         |
| `reset_performance_key` | Performance key = original key              |
| `clear_song`            | Close the viewer session (library unchanged) |

`SongSessionView` includes display chord symbols already spelled for the performance key, plus written symbols, lyric indices, warnings, semitone offset, and optional setlist context (Phase 8).

Written tokens and `source.originalContent` are unchanged when the key changes (`Song::display_chord`).

## Rendering

Each line is split at chord `lyricIndex` values (Unicode scalars) into syllable columns: chord above, lyric slice below. Segments wrap on narrow screens so a chord stays attached to its lyric slice.

Unrecognized / partial chords are underlined and keep their original text.

## Transpose

- `−` / `+` move the performance key by one semitone using `Key::transpose_semitones` (common spellings: `G+1 → Ab`).
- The key dropdown sets performance key directly.
- Reset restores the original key.
- If original key is missing, the first transpose infers it from the first fully recognized chord (minor quality → minor key, else major), otherwise `C`.

## Display preferences (UI-only)

Theme (dark / light / system) and independent lyric / chord / section sizes live in `localStorage`. They are not song data.

Dark is the default (live-performance first). System follows `prefers-color-scheme` unless Dark or Light is pinned.

## Out of scope

- Live mode (Phase 9)
- Web URL import, MusicXML
