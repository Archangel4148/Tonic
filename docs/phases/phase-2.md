# Phase 2 — Music Theory & Chord Engine

**Status:** Implemented, pending review  
**Do not start Phase 3 until explicitly instructed.**

## Goal

Build a reliable, UI-independent music engine.

## What shipped

- Notes, accidentals, pitch classes, and enharmonic equality vs spelling
- Semitone intervals independent of spelling
- Major/minor keys, including enharmonic key pairs (`C#` vs `Db`)
- Key-aware spelling (`Ab` in Ab major, not `G#`)
- Structured chord model (root, quality, seventh, extensions, alterations, sus, adds, bass)
- Extensible chord parser with fully / partially / unrecognized status
- Semitone transposition and source-key → target-key transposition
- Slash chords; complex extensions/alterations are preserved
- Capo sounding vs played helpers
- Exhaustive unit tests plus spec acceptance examples
- Docs: [`../music-theory.md`](../music-theory.md)

No transpose UI or Tauri commands were added. The shell only shows that the product is in phase 2.

## Acceptance criteria

| Criterion                     | Result                               |
| ----------------------------- | ------------------------------------ |
| `C → D`                       | `transpose_semitones(..., 2)`        |
| `Cm → Dm`                     | same                                 |
| `F# → G#`                     | same                                 |
| `Bb → C`                      | same                                 |
| `G/B → A/C#`                  | same                                 |
| `F#m7b5/C# → G#m7b5/D#`       | same                                 |
| Valid and invalid parse cases | fully / partial / unrecognized tests |

## Review notes

- Engine API: [`../music-theory.md`](../music-theory.md)
- Architecture still: UI → Tauri → app → domain / persist
- Domain still has zero crate dependencies (`cargo tree -p tonic-domain`)
- Do not wrap this engine in UI until Phase 5 (song viewer transpose controls)

## Known limitations

- No song document yet (Phase 3)
- Parser does not cover every jazz shorthand (`C2`, Nashville numbers, `alt`)
- German `H` is unrecognized
- Canonical symbols prefer ASCII (`m7b5` not `ø`, `maj7` not `Δ`)
- Capo frets above 12 are rejected
- Transposition is not exposed over IPC

## How to review

```bash
cargo test -p tonic-domain
cargo test --workspace
npm test
```
