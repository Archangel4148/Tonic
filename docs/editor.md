# Song editor (Phase 7)

Users can create a chart from scratch and correct imported songs. The UI edits metadata plus a **chord-over-lyrics** plaintext chart. Rust parses chords; the UI does not.

## Ownership

`AppServices` holds an **editor draft** in memory. It is not written to the library until **Save**. **Cancel** discards the draft.

| Action    | Result                                                                    |
| --------- | ------------------------------------------------------------------------- |
| New song  | Draft `Untitled` song, one Verse, empty chart. Not in the library yet.    |
| Edit song | Draft copy of the library song, exported as chord-over-lyrics.            |
| Save      | Parse the chart if needed, then write-through persist.                    |
| Cancel    | Drop draft. New songs are not kept. Existing songs stay as last saved.    |

Unsaved drafts do not survive app restart. That is intentional save/cancel behavior.

## Chart format

Alternating lines: chords on one line, lyrics on the next. Spaces place chords over words. Section headers look like `[Verse]`, `[Chorus 2]`, or `[Hook]`. Chord-only lines (intros) are allowed.

## What Rust does

- Plain-text import for the chart body (`editor_parse_body`)
- `parse_chord` on every written symbol (fully / partial / unrecognized)
- Persist only on Save

Written chord tokens stay authoritative. Transpose is disabled while the editor is dirty.

## IPC

| Command              | Purpose                                       |
| -------------------- | --------------------------------------------- |
| `editor_create`      | New unsaved song                              |
| `editor_begin`       | Edit a library song                           |
| `editor_state`       | Current draft, or `null`                      |
| `editor_save`        | Commit draft → library + session              |
| `editor_cancel`      | Discard draft                                 |
| `editor_update_meta` | Title, artist, key, tempo, meter, notes, tags |
| `editor_parse_body`  | Replace sections from chord-over-lyrics text  |

MusicXML songs keep `Song.score` on the draft. Chart edits do not clear an existing score. There is no notation editor.

## Out of scope here

- MusicXML authoring / staff editing
