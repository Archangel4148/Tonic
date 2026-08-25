# Tonic documentation

Tonic is a local-first musician's songbook: import, edit, transpose, organize, and perform chord charts offline.

This folder is the project documentation. The product requirements live in [`spec.md`](./spec.md). End users looking for installers should start at the [GitHub Releases page](https://github.com/Archangel4148/Tonic/releases/latest) or [`release.md`](./release.md).

## Start here

| Document                                      | What it covers                                   |
| --------------------------------------------- | ------------------------------------------------ |
| [Release](./release.md)                       | Installers, APK, versioning, update strategy     |
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
| [Accessibility](./accessibility.md)           | Focus, SR labels, contrast, data-loss guards     |

## Source of truth

[`spec.md`](./spec.md) is the product and engineering specification.

Historical build checkpoints (phases 1–12) live under [`phases/`](./phases/).

