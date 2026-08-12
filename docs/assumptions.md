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
- MusicXML uses `SourceFormat::MusicXml`. The canonical body is `Song.score`, not chart tokens.

## Editor (Phase 7)

- Editor draft lives only in `AppServices` until Save. Cancel drops it. New unsaved songs never hit disk.
- Chord tagging sends the symbol to Rust (`parse_chord`). The UI does not parse.
- “Paste chart to replace body” reuses `tonic-import` and keeps existing metadata when already filled in.
- Changing original key in the editor also updates performance key when they still matched.

## Setlists (Phase 8)

- Entries store `songId` only. The same song may appear multiple times via distinct `entry-{n}` ids.
- Per-entry performance key, capo (`0..=12`), and notes do not mutate the library `Song`.
- Opening an entry builds a display clone; transpose/key/reset in that session write the entry, not the song. Original key may still be inferred on the song if it was missing.
- Capo is not a domain `Song` field. Played key is derived (`played_key`) for the viewer banner.
- Missing referenced songs remain as entries and show `(missing song)`.
- Duplicate setlist copies entry settings but mints new setlist and entry ids.

## Live mode (Phase 9)

- Live mode is UI-only. It does not copy songs or become a second session owner.
- Forced dark theme and live type scale do not overwrite the user’s editor theme / type-scale prefs.
- Auto-scroll speed and hide-info persist in `localStorage`.
- Setlist next/prev skip missing songs and do not wrap.
- Keep-awake uses the Screen Wake Lock API when available; otherwise live mode still runs.
- A single song can enter live mode; previous/next stay disabled without a setlist.
- Live **Lock** (`L`) hides on-screen chrome but leaves hotkeys active.

## MusicXML (Phase 10)

- Scores are a separate model (`Score`). Optional lyric/harmony companion lines exist only for search and a simple chart readout.
- Sheet display uses the original MusicXML/MXL document. `Score` is for search/theory; transpose rewrites pitches in the original XML rather than rebuilding a simplified score.
- MXL is unzipped in Rust. The UI only sends bytes / text.
- OpenSheetMusicDisplay engraves only; it is not a music-theory engine.
- Unsupported MusicXML features warn and still show the supported notes.
- No notation authoring. Editor keeps the score when editing metadata or the companion chart.
- Pitch-class key jumps use the nearest signed interval (−6..=6) so sheet transpose does not leap an octave.

## Accessibility & polish (Phase 11)

- Skip link, focus-visible rings, larger hit targets, `prefers-contrast` / `prefers-reduced-motion`.
- Dirty editor or Details form triggers `beforeunload` and leave confirms.
- Library search is debounced; OSMD loads lazily when a score is shown.
- Boot failure offers Retry; filtered library empty copy differs from a blank songbook.
