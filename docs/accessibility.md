# Accessibility & polish (Phase 11)

Tonic aims to be usable on stage and at a desk: keyboard-first where it matters, readable under strong contrast preferences, and hard to lose work by accident.

## Keyboard & focus

- Skip link → `#main-content`
- Visible `:focus-visible` rings on interactive controls
- Live mode keeps its hotkey set (see [`live-mode.md`](./live-mode.md)); locked chrome is `inert`
- **Fullscreen** header control, plus `F11` / `Alt+Enter` in the main shell and live mode
- Library Songs / Setlists tabs expose `aria-controls` and tabpanels

## Screen readers

- Landmarks: header, main, footer, library aside
- Theme preference uses `aria-pressed`
- Chart lines: aria-label includes lyrics plus chord list; unrecognized / partial chords announce status
- Setlist Up / Down / Remove include the song title in the accessible name
- Polite live region announces “Working…” while IPC is busy

## Visual / touch

- Independent lyric / chord / section type scale (existing)
- Dark / light / system themes (existing)
- `prefers-contrast: more` strengthens borders and text
- `prefers-reduced-motion: reduce` disables decorative transitions
- Primary actions, chips, and icon buttons aim for ≥44×44 CSS px hit areas

## Reliability

| Situation                       | Behavior                                            |
| ------------------------------- | --------------------------------------------------- |
| Dirty editor or Details form    | `beforeunload` warning; leave flows confirm discard |
| Engine boot failure             | Alert + **Retry connection**                        |
| Empty library / filtered search | Distinct empty copy                                 |
| Sheet music loading             | Status text while OSMD renders                      |
| Destructive delete / remove     | Confirm + danger styling                            |

## Performance

- Library search IPC is debounced (~200 ms)
- OpenSheetMusicDisplay is dynamically imported only when a score is shown

## Out of scope

- Automated WCAG color metering
