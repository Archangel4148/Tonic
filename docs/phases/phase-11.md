# Phase 11 — Accessibility, Polish & Reliability

**Status:** Complete  
**Next:** Phase 12 (packaging) — implemented separately.

## Goal

Make the application production-quality for everyday rehearsal and gig use.

## What shipped

- Skip link to main content; landmarks and polite busy live region
- Global `:focus-visible`, larger touch targets (~44px), `prefers-contrast`, `prefers-reduced-motion`
- Theme chips `aria-pressed`; library tabs `aria-controls` / tabpanels
- Chart lines expose chords + recognition status to screen readers
- Live chrome uses `inert` when locked/hidden so it is not tabbable
- Sheet music loading status; OSMD loaded on demand
- Debounced library search
- Boot **Retry connection**; distinct empty / filter-empty copy
- `beforeunload` when the editor or Details form is dirty; Details dirty confirm
- Destructive setlist/song actions use danger styling + clearer labels
- Product phase reported by `AppServices` is **11**
- Docs: this report; README / architecture / testing updates

## Acceptance criteria

| Criterion                                                                          | Result                                                                                      |
| ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| No known critical workflows require unreliable behavior or unnecessary interaction | Dirty leave guards, boot retry, clear empty/error/loading states, keyboard/live a11y polish |

## Review notes

- Accessibility is layered on the existing React shell; domain/IPC boundaries unchanged.
- High contrast follows `prefers-contrast: more` rather than a separate theme preset.
- OSMD still mocks in jsdom; production loads it lazily only when a score is shown.

## Known limitations

- No automated color-contrast metering (manual gig-mode pass recommended)
- ChordPro import polish still deferred

## How to review

```bash
npm run tauri dev
npm test
```

Tab through the shell (skip link, library tabs, import, transpose). Toggle OS high contrast / reduced motion if available. Edit a song, then try closing the window — the browser/webview should warn. Open MusicXML and confirm “Rendering sheet music…” appears briefly. Disconnect the engine (browser-only `npm run dev`) and use **Retry connection**.
