# Song editor (Phase 7)

Users can create a chart from scratch and correct imported songs. Editing operates on the canonical `Song` model. The UI never parses chords or transposes.

## Ownership

`AppServices` holds an **editor draft** in memory. It is not written to the library until **Save**. **Cancel** discards the draft.

| Action    | Result                                                                    |
| --------- | ------------------------------------------------------------------------- |
| New song  | Draft `Untitled` song, one Verse, one empty line. Not in the library yet. |
| Edit song | Draft copy of the library song.                                           |
| Save      | Write-through persist; song can be reopened later.                        |
| Cancel    | Drop draft. New songs are not kept. Existing songs stay as last saved.    |

Unsaved drafts do not survive app restart. That is intentional save/cancel behavior.

## What the UI may do

- Type lyrics, metadata, tags, annotations
- Ask Rust to tag / retag / untag chords at a lyric index
- Add / rename / reorder / remove sections and lines
- Paste ChordPro or plain text to **replace the chart body** (parser correction)

## What Rust does

- `parse_chord` on every tagged symbol (fully / partial / unrecognized)
- Section labels via `SectionLabel::parse`
- Import parsers for “paste to replace body”
- Persist only on Save

Written chord tokens stay authoritative. Transpose is disabled while the editor is dirty.

## IPC

| Command                                                                                             | Purpose                                       |
| --------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| `editor_create`                                                                                     | New unsaved song                              |
| `editor_begin`                                                                                      | Edit a library song                           |
| `editor_state`                                                                                      | Current draft, or `null`                      |
| `editor_save`                                                                                       | Commit draft → library + session              |
| `editor_cancel`                                                                                     | Discard draft                                 |
| `editor_update_meta`                                                                                | Title, artist, key, tempo, meter, notes, tags |
| `editor_add_section` / `editor_set_section_label` / `editor_remove_section` / `editor_move_section` | Sections                                      |
| `editor_add_line` / `editor_remove_line` / `editor_set_lyrics`                                      | Lines                                         |
| `editor_tag_chord` / `editor_untag_chord` / `editor_replace_chord` / `editor_set_chord_index`       | Chord tagging + correction                    |
| `editor_set_annotation`                                                                             | Line annotation                               |
| `editor_parse_body`                                                                                 | Replace sections from pasted chart            |

MusicXML songs keep `Song.score` on the draft. Paste-to-replace-body may attach a score when the paste is MusicXML; chart-only pastes do not clear an existing score. There is no notation editor.

## Out of scope here

- MusicXML authoring / staff editing
