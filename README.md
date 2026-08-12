# Tonic

A local-first musician's songbook for importing, editing, transposing, and performing chord charts.

Tonic is built with **Tauri 2**, a **Rust** domain layer, and a **React + TypeScript** UI. It targets Android (sideloaded APK) and desktop (Windows, macOS, Linux). iOS is out of scope.

## Current phase

**Phase 3 — Canonical Song Model**

Songs are structured documents (sections, lines, chord/lyric tokens) in Rust. Import, library, editor, and live mode are not implemented yet.

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
- [Phase 1 report](./docs/phases/phase-1.md)
- [Phase 2 report](./docs/phases/phase-2.md)
- [Phase 3 report](./docs/phases/phase-3.md)

## License

MIT (see `docs/assumptions.md` if this should change before release)
