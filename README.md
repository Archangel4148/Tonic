# Tonic

A local-first musician's songbook for importing, editing, transposing, and performing chord charts.

Tonic is built with **Tauri 2**, a **Rust** domain layer, and a **React + TypeScript** UI. It targets Android (sideloaded APK) and desktop (Windows, macOS, Linux). iOS is out of scope.

## Current phase

**Phase 12 — Packaging & Release**

Version **1.0.0**. One desktop installer per OS; one Android APK for sideload. See [`docs/release.md`](./docs/release.md).

## Quick start

```bash
npm install
npm run tauri dev
```

```bash
npm test
npm run lint
```

## Install packaged builds

- Windows: run the NSIS `*-setup.exe` (embeds WebView2 for blank PCs)
- Android: sideload the universal `.apk`
- Details: [`docs/release.md`](./docs/release.md)

## Documentation

All project documentation lives in [`docs/`](./docs/README.md).

- [Product spec](./docs/spec.md)
- [Architecture](./docs/architecture.md)
- [Development](./docs/development.md)
- [Release & installation](./docs/release.md)
- [Music theory](./docs/music-theory.md)
- [Song model](./docs/song-model.md)
- [Import](./docs/import.md)
- [Viewer](./docs/viewer.md)
- [Persistence](./docs/persist.md)
- [Editor](./docs/editor.md)
- [Setlists](./docs/setlists.md)
- [Live mode](./docs/live-mode.md)
- [MusicXML](./docs/musicxml.md)
- [Accessibility](./docs/accessibility.md)
- [Phase 1 report](./docs/phases/phase-1.md)
- [Phase 2 report](./docs/phases/phase-2.md)
- [Phase 3 report](./docs/phases/phase-3.md)
- [Phase 4 report](./docs/phases/phase-4.md)
- [Phase 5 report](./docs/phases/phase-5.md)
- [Phase 6 report](./docs/phases/phase-6.md)
- [Phase 7 report](./docs/phases/phase-7.md)
- [Phase 8 report](./docs/phases/phase-8.md)
- [Phase 9 report](./docs/phases/phase-9.md)
- [Phase 10 report](./docs/phases/phase-10.md)
- [Phase 11 report](./docs/phases/phase-11.md)
- [Phase 12 report](./docs/phases/phase-12.md)

## License

MIT (see `docs/assumptions.md` if this should change before release)
