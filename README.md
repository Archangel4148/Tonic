# Tonic

A local-first musician's songbook for importing, editing, transposing, and performing chord charts.

Tonic is built with **Tauri 2**, a **Rust** domain layer, and a **React + TypeScript** UI. It targets Android (sideloaded APK) and desktop (Windows, macOS, Linux). iOS is out of scope.

## Current phase

**Phase 1 — Foundation & Architecture**

The app launches a shell and proves that domain, application, UI, and persistence boundaries are in place. Music theory, import, library, and live mode are not implemented yet.

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
- [Phase 1 report](./docs/phases/phase-1.md)

## License

MIT (see `docs/assumptions.md` if this should change before release)
