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

Filesystem JSON under the Tauri app data directory (`library/index.json` + `library/songs/{id}.json`). SQLite was deferred: serde `Song` already round-trips, files are inspectable, and a `SongLibrary` trait keeps a later swap possible. Favorite, tags, and recents are stored beside `Song`, not on the domain document.

## Import crate layout

ChordPro and plain-text parsers live in a dedicated `tonic-import` crate. They are not part of `tonic-domain` (no parsing of source formats in the music engine) and not part of `tonic-persist` (import is not storage). `tonic-app` orchestrates import and owns the current session song. IPC/UI arrived in Phase 5.

## Import behavior (Phase 4)

- Import never hard-fails; warnings plus a usable `Song`.
- User-facing summary for any warning: “Some content could not be recognized.”
- Unknown chords are preserved with `ParseStatus::Unrecognized`.
- Chord-line detection requires a majority of **fully** recognized tokens (partial words such as `Amazing` do not count).
- Plain `[Chorus]` is a section header, not ChordPro.
- `{capo}` and dump markers like `[(capo][+1)]` stay in-line as annotations (`Capo 1` / `Capo +1`), not a domain capo field. `{capo}` also copies into song notes.
- ChordPro `#` lines are ignored. `{artist}` wins over `{composer}` / `{subtitle}`.
- `{new_song}` / `{ns}` only skips remaining content if a song body was already parsed. Leading `{ns}` in `.pro` dumps is ignored.
- Default section is Verse when none is declared.
- `SongId` is still assigned by the caller.

## Tauri opener plugin

`tauri-plugin-opener` remains from the official template even though Phase 1 does not call it.

## Themes

Dark is the default (stage-friendly). The user can pin Dark, Light, or System. System follows `prefers-color-scheme`. Theme and type scale stay in `localStorage` (presentation only, not song data).

## Viewer / transpose (Phase 5)

- The UI renders `SongSessionView`; it does not reparse chart text.
- `−`/`+` accumulate a semitone offset from the original key (`Key::transpose_semitones` preferred spellings: `G+1 → Ab`).
- Missing original key is inferred on first transpose from the first fully recognized chord (minor → minor key), else `C`.
- Theme and type scale are presentation state, not song data.
- Session song is discarded when the process exits. (Superseded in Phase 6: import saves to the library; closing the viewer does not delete the song.)

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
- Phase 5 exposes transposition over IPC; the UI never transposes locally.

## Song model (Phase 3)

- Written chord tokens are authoritative; performance-key display is derived.
- `SongId` is an opaque string assigned outside the domain crate.
- `lyric_index` counts Unicode scalars in concatenated lyric tokens.
- Time-signature denominators must be powers of two; tempo is 1–400 BPM.
- JSON is the Phase 3 interchange format and, as of Phase 6, also the on-disk library format.
- MusicXML is not a `SourceFormat` variant yet (use `other` if needed until Phase 10).
