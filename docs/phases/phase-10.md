# Phase 10 — MusicXML & Sheet Music

**Status:** Complete  
**Phase 11 may proceed.**

## Goal

Expand beyond text-based chord charts.

## What shipped

- Domain `Score` model (parts, measures, notes/rests/harmony) distinct from chart tokens
- `SourceFormat::MusicXml` and `Song.score`
- MusicXML + MXL import in `tonic-import` (`roxmltree` + `zip`)
- Sheet display uses original MusicXML/MXL engraving (grand staff, both clefs, beams, dynamics); transpose rewrites pitches in that document
- OpenSheetMusicDisplay in the viewer and live mode
- Import panel: MusicXML paste, `.musicxml` / `.mxl` files, sample score
- MusicXML-specific warnings (`UnsupportedFeature`); supported notes still render
- Docs: [`../musicxml.md`](../musicxml.md)
- Product phase reported by `AppServices` is **10**

No web URL import, no notation authoring, no Phase 11 polish.

## Acceptance criteria

| Criterion                                                               | Result                                                                                          |
| ----------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Supported MusicXML documents render correctly and remain usable offline | Import → score + OSMD sheet; original XML on `Song.source`; works without network after install |

## Review notes

- UI still never parses or transposes. OSMD engraves the original MusicXML (pitch rewrite only when the key changes).
- Scores are not forced into the chord-chart model. Companion lyric/harmony lines are optional for search.
- MXL unzip happens in Rust, not in the WebView.
- Key picker / reopen uses nearest signed pitch-class delta (−6..=6) so sheet transpose does not jump an octave.

## Known limitations

- Companion chart extraction still skips advanced notation with a warning; the **sheet** keeps those markings from the original XML
- OSMD is mocked in jsdom tests
- Editor cannot author notation; it only edits metadata / extracted chart
- ChordPro import polish still deferred

## How to review

```bash
npm run tauri dev
npm test
```

In the app: Import → **Twinkle (MusicXML)** (or open a `.musicxml` / `.mxl` file). Confirm the staff renders, transpose changes written pitches in the sheet (not the stored source), and the song still opens offline after reload.
