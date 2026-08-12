# Assumptions

Decisions made where the spec left room. Each one uses the simplest behavior consistent with the product.

## Product name and identity

- App name: **Tonic**
- Bundle identifier: `com.tonic.app`
- Crate prefix: `tonic-*`
- The crates.io gRPC crate `tonic` is not used

## UI framework

React + TypeScript is the frontend. The spec allowed React, Svelte, Vue, or vanilla web components. React was selected by the product owner.

## Domain vs persistence dependency

The spec's layer diagram is interpreted as layering, not as domain depending on persistence. `tonic-domain` has zero dependencies. `tonic-app` depends on domain and persist. Persistence will depend on domain types once those types exist.

## Persistence technology

Not chosen yet. Phase 1 only has an in-memory `Store` stub. SQLite vs filesystem vs another local store is a Phase 6 decision and will be documented then.

## Import crate layout

Phase 4 will decide whether ChordPro / plain-text parsers live in `tonic-persist`, `tonic-domain`, or a new `tonic-import` crate. No import crate was created in Phase 1.

## Tauri opener plugin

`tauri-plugin-opener` remains from the official template even though Phase 1 does not call it.

## Themes

The Phase 1 shell is dark-first and still follows `prefers-color-scheme` for light mode. Dedicated theme settings and live-performance dark mode arrive later.

## Android

Android is a required eventual target, but Phase 1 does not generate the Android project or install mobile toolchains. Desktop launch satisfies the Phase 1 "application launches" criterion.

## Documentation location

Engineering docs live under `docs/`. The product spec was moved to `docs/spec.md` so documentation has a single home.

## Versioning

The workspace starts at `0.1.0`. Release versioning and update strategy are Phase 12.

## License

Workspace crates declare MIT for now. This can change if the product owner picks a different license before release.
