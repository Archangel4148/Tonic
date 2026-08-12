# Tonic documentation

Tonic is a local-first musician's songbook: import, edit, transpose, organize, and perform chord charts offline.

This folder is the project documentation. The product requirements live in [`spec.md`](./spec.md).

## Start here

| Document                                      | What it covers                                   |
| --------------------------------------------- | ------------------------------------------------ |
| [Development](./development.md)               | Install, run, test, lint, and build              |
| [Technology choices](./technology-choices.md) | Why Tauri, Rust, React, and the supporting tools |
| [Architecture](./architecture.md)             | Layer boundaries and dependency rules            |
| [Project structure](./project-structure.md)   | Where code and docs live                         |
| [State management](./state-management.md)     | What owns authoritative vs derived state         |
| [Testing](./testing.md)                       | Rust and UI test setup                           |
| [Assumptions](./assumptions.md)               | Decisions made where the spec was open           |
| [Music theory](./music-theory.md)             | Notes, keys, parser, transposition, spelling     |
| [Song model](./song-model.md)                 | Canonical song, tokens, JSON interchange         |
| [Import](./import.md)                         | ChordPro and plain-text parsers                  |
| [Viewer](./viewer.md)                         | Song reading, alignment, transpose UI            |
| [Persistence](./persist.md)                   | Local JSON library and setlists                  |
| [Editor](./editor.md)                         | New song, chart editor, save/cancel              |
| [Setlists](./setlists.md)                     | Ordered song references, per-entry overrides     |
| [Live mode](./live-mode.md)                   | Stage view, auto-scroll, setlist navigation      |
| [MusicXML](./musicxml.md)                     | Score import, MXL, OSMD sheet rendering          |
| [Phase 1 report](./phases/phase-1.md)         | Foundation checkpoint                            |
| [Phase 2 report](./phases/phase-2.md)         | Music-engine checkpoint                          |
| [Phase 3 report](./phases/phase-3.md)         | Song-model checkpoint                            |
| [Phase 4 report](./phases/phase-4.md)         | Import checkpoint                                |
| [Phase 5 report](./phases/phase-5.md)         | Viewer checkpoint                                |
| [Phase 6 report](./phases/phase-6.md)         | Library checkpoint                               |
| [Phase 7 report](./phases/phase-7.md)         | Editor checkpoint                                |
| [Phase 8 report](./phases/phase-8.md)         | Setlists checkpoint                              |
| [Phase 9 report](./phases/phase-9.md)         | Live mode checkpoint                             |
| [Phase 10 report](./phases/phase-10.md)       | MusicXML / sheet music checkpoint                |

## Source of truth

[`spec.md`](./spec.md) is the current product and engineering specification. Implement only the requested phase, then stop at that phase's review checkpoint.
