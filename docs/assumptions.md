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

The spec's layer diagram is interpreted as layering, not as domain depending on persistence. `tonic-domain` depends only on `serde` / `serde_json` (song interchange). It has no Tauri or UI dependencies. `tonic-app` depends on domain and persist. Persistence will depend on domain `Song` types in Phase 6.

## Persistence technology

Not chosen yet. Phase 1 only has an in-memory `Store` stub. SQLite vs filesystem vs another local store is a Phase 6 decision and will be documented then.

## Import crate layout

ChordPro and plain-text parsers live in a dedicated `tonic-import` crate. They are not part of `tonic-domain` (no parsing of source formats in the music engine) and not part of `tonic-persist` (import is not storage). `tonic-app` orchestrates import; IPC/UI arrives in Phase 5.

## Import behavior (Phase 4)

- Import never hard-fails; warnings plus a usable `Song`.
- User-facing summary for any warning: “Some content could not be recognized.”
- Unknown chords are preserved with `ParseStatus::Unrecognized`.
- Chord-line detection requires a majority of **fully** recognized tokens (partial words such as `Amazing` do not count).
- Plain `[Chorus]` is a section header, not ChordPro.
- `{capo}` / `Capo:` become song notes, not a domain capo field.
- `{new_song}` skips remaining content rather than splitting files into multiple songs.
- Default section is Verse when none is declared.
- `SongId` is still assigned by the caller.

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

## Music theory (Phase 2)

- Bare `sus` means `sus4`.
- `ø` / `hdim` canonicalizes to `m7b5`.
- `M` vs `m` after the root is case-sensitive: `CM7` is major seven, `Cm7` is minor seven.
- German `H` is unrecognized.
- Double sharp is `##` / `𝄪` only; LilyPond `x` is not accepted so tokens like `Cxyz` stay partial rather than `C##`.
- Keyless transpose preserves accidental family; natural notes use the sharp chromatic (`C+1 → C#`, `B+2 → C#`).
- Minor keys use the natural minor scale for diatonic spelling.
- Capo frets are `0..=12`.
- Phase 2 does not expose transposition over IPC or in the UI.

## Song model (Phase 3)

- Written chord tokens are authoritative; performance-key display is derived.
- `SongId` is an opaque string assigned outside the domain crate.
- `lyric_index` counts Unicode scalars in concatenated lyric tokens.
- Time-signature denominators must be powers of two; tempo is 1–400 BPM.
- JSON is the Phase 3 interchange format, not the library database.
- MusicXML is not a `SourceFormat` variant yet (use `other` if needed until Phase 10).
