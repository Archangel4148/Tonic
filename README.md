# Tonic

A local-first musician's songbook for importing, editing, transposing, and performing chord charts.

Tonic is built with **Tauri 2**, a **Rust** domain layer, and a **React + TypeScript** UI. It targets Android (sideloaded APK) and desktop (Windows, macOS, Linux). iOS is out of scope.

## Current phase

**Phase 5 — Song Viewer**

Import a ChordPro or plain-text chart, read aligned chords and lyrics, and change key. Library save, editor, and live mode are not implemented yet.

## Quick start

```bash
npm install
npm run tauri dev
```

```bash
npm test
npm run lint
```

## Documentation

All project documentation lives in [`docs/`](./docs/README.md).

- [Product spec](./docs/spec.md)
- [Architecture](./docs/architecture.md)
- [Development](./docs/development.md)
- [Music theory](./docs/music-theory.md)
- [Song model](./docs/song-model.md)
- [Import](./docs/import.md)
- [Viewer](./docs/viewer.md)
- [Phase 1 report](./docs/phases/phase-1.md)
- [Phase 2 report](./docs/phases/phase-2.md)
- [Phase 3 report](./docs/phases/phase-3.md)
- [Phase 4 report](./docs/phases/phase-4.md)
- [Phase 5 report](./docs/phases/phase-5.md)

## License

MIT (see `docs/assumptions.md` if this should change before release)
