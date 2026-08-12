# Chord Transposer & Musician's Songbook

## Software Requirements Specification

**Document status:** Agent-ready product and engineering specification  
**Version:** 1.0  
**Purpose:** This document is the authoritative implementation specification for an agentic coding model. Development is intentionally phased so each milestone can be reviewed before the next is started.

---

# 1. Product Vision

Build a cross-platform application for musicians to **import, manage, edit, transpose, and perform songs containing chord charts and/or sheet music**.

The application's core workflow is:

> **Import → Normalize → Edit → Transpose → Organize → Perform**

The application should feel like a musician's dedicated digital songbook rather than a generic text editor.

The primary live-performance goals are:

- Readability at a glance
- Extremely fast key changes
- Minimal interaction while performing
- Reliable offline operation
- Predictable chord/lyric alignment
- Fast navigation through setlists
- Music-theory-accurate transposition

The application should support both simple chord sheets and progressively more sophisticated musical notation without making the basic chord-chart workflow unnecessarily complicated.

---

# 2. Goals

## 2.1 Primary Goals

1. Provide accurate chromatic chord transposition.
2. Support common and complex chord notation.
3. Preserve chord-to-lyric relationships during rendering.
4. Import songs from multiple common formats.
5. Allow musicians to edit imported songs.
6. Provide a searchable local song library.
7. Allow songs to be organized into setlists.
8. Provide a dedicated live-performance mode.
9. Work fully offline after installation/data import.
10. Provide accessible, readable layouts on mobile and desktop.
11. Preserve musical meaning when transposing, including enharmonic spelling.
12. Keep the music-theory/domain layer independent of the UI.

## 2.2 Secondary Goals

- Support capo-aware arrangements.
- Support sheet music through MusicXML.
- Provide annotations and song metadata.
- Provide configurable display preferences.
- Make the application suitable for rehearsal as well as performance.
- Make the data model extensible for future musical features.

---

# 3. Non-Goals

The following are explicitly outside the initial product scope unless a later phase specifically adds them:

- Full DAW functionality
- Audio recording/editing
- Full notation authoring comparable to professional notation software
- Audio-to-chord transcription
- Automatic song identification
- Cloud synchronization
- User accounts
- Social networking
- Collaborative editing
- Streaming music services
- Copyrighted song-content discovery or scraping
- Replacing a professional sheet-music engraving application

The architecture should not prevent future additions, but these features must not drive the MVP design.

---

# 4. Target Users

The primary user is a musician who needs to keep chord charts available during rehearsal or a live performance.

Typical workflows include:

- Importing an existing ChordPro file
- Pasting a chord chart
- Quickly changing the key for a singer
- Building a setlist before a gig
- Reading the setlist on a phone/tablet
- Using auto-scroll while playing
- Returning to a song later with its preferred arrangement preserved

---

# 5. Target Platforms

## 5.1 Primary Target

Android: The sole mobile target for live gigging. Distribution will be handled directly via .apk file (sideloading).

## 5.2 Secondary Targets

- Windows
- macOS
- Linux

(Note: iOS and iPadOS are explicitly out of scope for the initial product phase.)

## 5.3 Technology Selection

The application must be built using Tauri (v2) to allow compilation to both desktop native installers and Android .apk files from a unified codebase.

To ensure performance and native compatibility across all platforms, the technology stack is strictly defined:

Backend / Domain Layer (Rust): The core music-theory engine, transposition algorithms, text parsing, file I/O, and local persistence must be written in Rust. This ensures the domain logic compiles directly into the application binary for both desktop and mobile.

Frontend / Presentation Layer (JavaScript/TypeScript): The UI, responsiveness, and rendering logic must be written in JS/TS. The implementation agent may choose a suitable UI framework (e.g., React, Svelte, Vue, or vanilla web components).

---

# 6. Core User Workflows

## 6.1 Import a Song

1. User chooses Import.
2. User selects a supported source format.
3. Application parses the source.
4. Application converts it into the canonical internal song model.
5. Application displays the imported song.
6. Any parsing problems are surfaced without destroying successfully parsed content.
7. User can save the song to the library.

## 6.2 Create a Song Manually

1. User selects New Song.
2. User enters title and optional metadata.
3. User enters or pastes chord/lyric content.
4. Application parses recognizable chords.
5. User can correct or manually tag ambiguous content.
6. User saves the song.

## 6.3 Transpose a Song

1. User opens a song.
2. User selects a target key or changes the semitone offset.
3. All recognized chords transpose immediately.
4. Lyrics and structural content remain unchanged.
5. Chord spelling follows the selected musical context.
6. The user's preferred key can be saved.

## 6.4 Build a Setlist

1. User creates a setlist.
2. User adds songs from the library.
3. User orders songs.
4. Individual setlist entries may optionally override the song's default performance key.
5. User starts performance mode.

## 6.5 Perform a Setlist

1. Application enters a minimal, distraction-free display.
2. Current song is rendered in a highly readable layout.
3. User can transpose without leaving performance mode.
4. User can navigate to the previous/next song.
5. Auto-scroll can be enabled.
6. Screen wake behavior should be configurable.
7. Setlist progress is visible but unobtrusive.

---

# 7. Architectural Principles

The application must maintain a clean separation between presentation, application state, domain logic, and persistence.

Recommended conceptual dependency direction:

```text
UI / Presentation
       ↓
Application State / Services
       ↓
Domain / Music Theory
       ↓
Persistence / Import / Export
```

## 7.1 Architectural Invariants

1. UI components must not contain music-theory algorithms.
2. Chord parsing must not depend on UI components.
3. Transposition must not depend on UI components.
4. Persistence must not become the authoritative representation of active application state.
5. A song must have one canonical in-memory representation.
6. Derived values must not be independently duplicated as authoritative state.
7. Imported source text should not be destroyed when normalized.
8. The renderer must consume the canonical model rather than reparsing raw text.
9. Setlists must reference songs rather than duplicate entire song documents.
10. Setlist-specific overrides must be represented explicitly.
11. The application must not create multiple competing chord-transposition implementations.
12. The domain layer must be independently unit-testable.

## 7.2 Agent Rule

Before implementing a feature, inspect the existing architecture and reuse existing abstractions where appropriate. Do not create parallel implementations merely because they are locally convenient.

---

# 8. Canonical Domain Model

The exact programming-language representation is implementation-dependent, but the conceptual model is mandatory.

## 8.1 Song

A Song should contain:

- Stable identifier
- Title
- Artist
- Album, if known
- Original key, if known
- Preferred/default performance key
- Tempo, if known
- Time signature, if known
- Sections
- Source information
- Optional notes
- Creation/update metadata

## 8.2 Song Source

The original imported representation should be preserved where practical.

Example:

```text
source:
    format: chordpro
    originalContent: ...
```

Normalization must not destroy the source merely because the user changes keys.

## 8.3 Section

Sections should support semantic labels such as:

- Intro
- Verse
- Pre-Chorus
- Chorus
- Bridge
- Instrumental
- Solo
- Outro
- Custom

The system must not require every song to use only predefined section names.

## 8.4 Lines and Tokens

Textual songs should be represented using structured tokens rather than raw strings alone.

Conceptually:

```text
Section
 └── Line
      ├── Chord token
      ├── Lyric token
      ├── Annotation token
      └── ...
```

The model must preserve enough positional information to render chords in their intended relationship to lyrics.

## 8.5 Chord

A Chord must represent musical components independently.

At minimum:

- Root note
- Quality
- Extensions
- Alterations
- Suspensions
- Added tones
- Bass note for slash chords
- Original textual representation where useful
- Parse status/confidence

For example:

```text
F#m7b5/C#
root: F#
quality: minor
extensions: 7
alterations: b5
bass: C#
```

Do not make the entire chord a single opaque string internally.

---

# 9. Music Theory Engine

The music-theory engine is a core domain subsystem and must be thoroughly unit tested.

## 9.1 Notes

Support the chromatic pitch classes and both common enharmonic spellings:

- A / A#
- Bb
- B
- C / B#
- C# / Db
- D
- D# / Eb
- E / Fb
- F / E#
- F# / Gb
- G
- G# / Ab

The implementation may internally use pitch classes, but rendering must retain the correct spelling.

## 9.2 Keys

Support major and minor keys and their common enharmonic equivalents.

Key context should influence chord spelling.

## 9.3 Intervals

The engine should represent semitone intervals independently from note spelling.

## 9.4 Chord Parsing

The parser should recognize at least:

```text
C
Cm
C#
Db
F#m

C7
Cm7
Cmaj7
C9
C11
C13
C6
Cm6

Cadd9
C7b5
C7#5
C7b9
C7#9
Cmaj7#11

Csus2
Csus4

Cdim
Caug

G/B
D/F#
F#m7b5/C#
```

The parser should be extensible rather than built as a fixed list of string comparisons.

## 9.5 Unrecognized Chords

Unrecognized or unusual chord text must not cause the entire song to fail.

The parser should distinguish between:

- Fully recognized
- Partially recognized
- Unrecognized

Unrecognized content should be preserved and rendered where possible.

## 9.6 Transposition

The transposition engine must support:

- Positive semitone offsets
- Negative semitone offsets
- Direct source-key → target-key transposition
- Major/minor context
- Slash chords
- Complex extensions and alterations

Only pitch-bearing components should change.

For example:

```text
C7b9
```

may become:

```text
D7b9
```

while preserving `7b9`.

## 9.7 Enharmonic Spelling

Enharmonic spelling must be context-aware.

The application should prefer musically sensible spellings based on the destination key rather than blindly choosing whichever spelling has the shortest string.

Example requirement:

> In a context where Ab is the appropriate scale degree, render Ab rather than G#.

The implementation should document its spelling strategy and encode representative cases in tests.

## 9.8 Capo

The application must distinguish:

- Concert/sounding key
- Chord shapes the musician plays
- Capo position

Example conceptual state:

```text
Sounding key: A
Capo: 2
Played shapes: G
```

The UI must make this distinction understandable.

---

# 10. Import & Export

## 10.1 ChordPro

Required support for:

- `.cho`
- `.crd`

Support common inline chord syntax:

```text
[C]Hello [G]world
```

The parser should preserve relevant ChordPro metadata and structural directives when possible.

## 10.2 Plain Text Chord-over-Lyrics

The application should recognize common layouts where chords appear above lyrics.

Example:

```text
C          G
Amazing grace how sweet
F             C
The sound that saved
```

The parser should use spacing/position information to associate chords with lyric locations.

Ambiguous input must remain editable rather than being destructively guessed.

## 10.3 MusicXML

MusicXML support is a later-phase requirement.

The system should eventually accept:

- `.musicxml`
- `.mxl`

MusicXML should be normalized into a suitable score representation rather than forced into the text-based chord-chart model.

## 10.4 Manual Editor

The editor must allow users to:

- Enter lyrics
- Enter chords
- Create sections
- Correct parser mistakes
- Add metadata
- Save changes

## 10.3 Web URL Import

The application should support importing songs directly from a user-provided URL to a supported guitar-tab, chord-chart, or lyrics website.

The goal is to make the common workflow:

```text
Find song online → Copy URL → Paste into application → Import
```

as fast and frictionless as possible.

### 10.3.1 User Workflow

1. User selects **Import from URL**.
2. User pastes a URL into the import field.
3. Application validates the URL.
4. Application determines whether the URL belongs to a supported website.
5. Application retrieves and parses the page using the appropriate website adapter.
6. Extracted content is converted into the canonical internal Song model.
7. Application displays an import preview.
8. User can inspect and correct the imported song.
9. User saves the result to the local library.

The imported song must subsequently behave identically to songs imported through ChordPro, plain text, or other supported formats.

### 10.3.2 Supported Websites

Website support must be implemented through explicit site-specific adapters.

The application must **not** assume that arbitrary webpages can be parsed reliably.

The importer architecture should conceptually resemble:

```text
WebImporter
├── SiteAdapterA
├── SiteAdapterB
├── SiteAdapterC
└── ...
```

Each adapter is responsible for extracting information from its supported website and converting it into the application's canonical Song model.

The initial implementation should support the specific guitar-tab/chord website(s) identified as the primary source by the product owner, with additional websites added through separate adapters.

Unsupported websites must fail gracefully and clearly indicate that the URL is not currently supported.

### 10.3.3 Extracted Information

Where available, the importer should attempt to extract:

- Song title
- Artist
- Album
- Original key
- Capo position
- Tempo
- Time signature
- Sections
- Lyrics
- Chords
- Chord-to-lyric positioning
- Song notes or annotations
- Other relevant musical metadata

The importer should preserve all successfully extracted information even if some fields are unavailable.

Missing metadata must not cause the import to fail.

### 10.3.4 Canonical Model Integration

Web import must use the same canonical Song model as every other importer.

The architecture should follow:

```text
Web URL
   ↓
Website Adapter
   ↓
Extracted Content
   ↓
Canonical Song Model
   ↓
Viewer / Editor / Transposer / Library
```

The viewer, editor, transposition engine, library, and setlist system must not need to know whether a song originated from a URL.

Web import must never create a separate song representation that bypasses the normal domain model.

### 10.3.5 Import Preview

Because webpage extraction can be less reliable than structured formats, imported content must be presented to the user for review before being committed to the library.

The preview should allow the user to identify:

- Missing lyrics
- Incorrect chords
- Incorrect chord placement
- Incorrect metadata
- Unsupported or unrecognized chords
- Incorrect section boundaries
- Other obvious extraction errors

The user must be able to edit the imported result before saving.

### 10.3.6 Parsing Failures

Web import must be resilient to incomplete or unexpected page content.

If some content can be extracted successfully, preserve that content rather than failing the entire import.

For example:

```text
Title:       Successfully extracted
Artist:      Successfully extracted
Lyrics:      Successfully extracted
Chords:      Partially extracted
Capo:        Not found
```

The application should still allow the user to continue editing the result.

If the page cannot be parsed at all, provide a clear error explaining that the page could not be imported.

### 10.3.7 Website Changes

Website adapters must be isolated from the rest of the application so that changes to a site's page structure do not require modifications to the core song model, editor, renderer, or music-theory engine.

A site-specific parsing failure should be contained to that adapter.

### 10.3.8 Network and Offline Behavior

URL import inherently requires network access and is therefore an online operation.

However, once successfully imported:

- The song must be stored locally.
- The song must be available offline.
- Viewing, editing, transposition, and performance mode must not require returning to the original website.
- The application must not depend on continued availability of the source webpage for normal use.

The application should not automatically re-fetch a source webpage merely to display or perform a previously imported song.

### 10.3.9 Source Metadata

When a song is imported from a URL, the application should retain source metadata where appropriate:

```text
source:
    format: web
    url: <original URL>
    website: <identified website>
```

The source URL is metadata about the imported content and must not replace the canonical Song representation.

### 10.3.10 Acceptance Criteria

Web URL import is considered complete when:

1. A user can paste a URL from at least one explicitly supported guitar-tab/chord website.
2. The application recognizes the supported website.
3. The appropriate site adapter retrieves and parses the page.
4. Song metadata is extracted where available.
5. Lyrics and chords are converted into the canonical Song model.
6. Chord-to-lyric relationships are preserved where the source provides sufficient information.
7. Unsupported or unrecognized chords are preserved rather than silently discarded.
8. The user receives an import preview before saving.
9. The imported song can be edited.
10. The imported song can be transposed using the normal music-theory engine.
11. The imported song can be saved to the library.
12. The saved song remains fully usable offline.
13. Unsupported URLs fail gracefully.
14. Network failures fail gracefully.
15. Website-specific parsing logic remains isolated from the rest of the application.
16. Automated tests cover representative supported pages and important parsing failure cases.

### 10.3.11 Future Extensions

Potential future enhancements include:

- Additional website adapters
- Automatic detection of supported URLs
- Importing multiple songs from a collection/setlist URL
- Re-import/update from the original source
- Browser share-sheet integration on mobile
- "Open in Chord Transposer" browser integration
- Clipboard-based URL detection

---

# 11. Rendering

## 11.1 Chord/Lyric Alignment

The renderer must preserve meaningful chord placement.

Do not simply render chords and lyrics as unrelated text blocks.

On narrow screens, content must reflow without making chords impossible to associate with the relevant lyrics.

## 11.2 Responsive Layout

The interface must support:

- Phone portrait
- Phone landscape
- Tablet
- Desktop

The live-performance layout should prioritize readable content over decorative UI.

## 11.3 Sheet Music

MusicXML rendering should eventually use an appropriate notation renderer such as VexFlow, OpenSheetMusicDisplay, or another suitable engine.

The implementation agent should select and document the library.

## 11.4 Themes

At minimum:

- Light theme
- Dark theme

Dark mode is a first-class live-performance feature.

## 11.5 Typography

Users must be able to independently adjust:

- Lyric size
- Chord size
- Section/header size

The application should maintain comfortable line spacing and avoid cramped presentation.

---

# 12. Song Library

The library should provide:

- Song list
- Search
- Favorites
- Tags
- Artist filtering
- Key filtering
- Recently opened
- Recently modified
- Sorting
- Delete/archive
- Duplicate/copy
- Import/export

Search should be fast enough for live use.

A user should be able to locate a song without navigating through multiple deep menus.

---

# 13. Setlists

Setlists should contain:

- Stable identifier
- Name
- Ordered song entries
- Optional notes
- Optional date/event metadata

Each entry should reference a song.

A setlist entry may override:

- Performance key
- Capo
- Notes
- Potentially other performance-specific settings

Example:

```text
Song:
    Original key: C
    Default key: C

Setlist entry:
    Performance key: Bb
    Capo: 2
```

The song itself should remain unchanged.

Setlists must support:

- Create
- Rename
- Duplicate
- Delete
- Add song
- Remove song
- Reorder
- Open song
- Start performance mode

---

# 14. Live Performance Mode

Live mode is a primary product feature.

It should provide:

- Fullscreen or near-fullscreen presentation
- Large readable text
- Minimal controls
- Dark stage-friendly theme
- Previous/next song
- Current setlist position
- Instant transpose
- Auto-scroll
- Scroll speed adjustment
- Manual scrolling
- Screen wake/keep-awake behavior where supported
- Optional hiding of metadata
- Fast return to normal editing mode

The controls should never obscure the song unnecessarily.

## 14.1 Navigation

The user should be able to move between songs with minimal interaction.

Where platform conventions permit, support:

- Tap controls
- Swipe gestures
- Keyboard shortcuts on desktop
- External keyboard/page-turner-friendly navigation where feasible

## 14.2 Auto-Scroll

Auto-scroll should provide:

- Start/stop
- Variable speed
- Visible speed control
- Reset to top
- Smooth scrolling
- No unexpected jumps

---

# 15. Editing

The editor should support:

- Title
- Artist
- Key
- Tempo
- Time signature
- Sections
- Lyrics
- Chords
- Notes
- Chord corrections
- Manual chord tagging

Editing must operate on the canonical model.

The UI should not directly mutate raw imported text as its primary state representation.

---

# 16. Persistence & Offline Operation

The application must function without an internet connection after installation and local data import.

Requirements:

- Songs persist locally.
- Setlists persist locally.
- User display preferences persist locally.
- Transposition preferences persist appropriately.
- Application must not lose song data when connectivity disappears.
- Live mode must not require network access.
- Imported songs must remain available offline.
- Data should be stored in a durable local format appropriate to the selected platform.

Possible technologies include:

- IndexedDB
- SQLite
- Local filesystem
- Platform-specific storage

The implementation agent should choose an appropriate mechanism and document it.

---

# 17. State Management

Application state must have a clear ownership model.

At minimum, distinguish:

### Authoritative state

Examples:

- Song documents
- Library entries
- Setlists
- User preferences

### Derived state

Examples:

- Current transposed chords
- Search results
- Setlist progress
- Rendered representations

Derived state should be recalculated from authoritative state rather than independently maintained in multiple places.

---

# 18. Error Handling

The application must degrade gracefully.

Examples:

### Malformed import

Show:

> Some content could not be recognized.

Preserve all usable content.

### Unknown chord

Preserve the original chord text and mark it as unrecognized.

Do not delete it.

### Unsupported MusicXML feature

Display the supported portion and provide a meaningful warning where practical.

### Storage failure

Do not silently discard edits.

### Network loss

The application should continue functioning normally for all offline-capable functionality.

---

# 19. Accessibility

Support:

- Adjustable font sizes
- High contrast
- Dark mode
- Touch-friendly controls
- Keyboard navigation on desktop
- Screen-reader-friendly labels where practical
- Controls that do not rely solely on color
- Adequate hit targets
- Clear focus states

Accessibility must not be treated as a final cosmetic pass.

---

# 20. Performance Requirements

## 20.1 Transposition

Changing a song's key should feel instantaneous.

Target:

**<200 ms** for ordinary song documents.

## 20.2 Navigation

Next/previous setlist song should feel immediate.

## 20.3 Rendering

Normal chord-chart rendering should not visibly stutter.

## 20.4 Startup

The application should avoid unnecessary startup work and should not load large libraries synchronously when it can avoid doing so.

---

# 21. Privacy & Data Ownership

The application should be local-first.

Unless a later phase explicitly introduces cloud services:

- User song data remains local.
- No account is required.
- No internet connection is required for core functionality.
- No telemetry is required for MVP functionality.

---

# 22. Testing Strategy

Testing is mandatory throughout development.

## 22.1 Domain Unit Tests

Must cover:

- Note parsing
- Enharmonic equivalents
- Key parsing
- Chord parsing
- Chord transposition
- Slash chords
- Extensions
- Alterations
- Minor keys
- Major keys
- Edge cases
- Invalid input

## 22.2 Import Tests

Use representative fixtures for:

- ChordPro
- Plain text
- Malformed input
- Mixed content
- Unrecognized chords

## 22.3 Rendering Tests

Test:

- Short songs
- Long songs
- Narrow screens
- Long chord names
- Dense chord sequences
- Sections
- Unrecognized chords

## 22.4 Persistence Tests

Test:

- Save
- Load
- Update
- Delete
- Restart/reload
- Setlist references
- Setlist-specific overrides

## 22.5 End-to-End Tests

At minimum, validate:

```text
Import → Parse → Save → Open → Transpose → Setlist → Performance Mode
```

## 22.6 Regression Requirement

Every bug discovered during development should result in a regression test when practical.

---

# 23. Development Phases

Each phase is independently reviewable.

The agent must not begin the next phase until explicitly instructed.

Every phase must leave the project buildable/runnable.

---

## Phase 1 — Foundation & Architecture

### Goal

Create the project skeleton and establish the architectural boundaries.

### Implement

- Project initialization
- Selected framework/toolchain
- Build/run configuration
- Basic application shell
- Domain/application/UI/persistence boundaries
- Initial state architecture
- Testing infrastructure
- Formatting/linting/static analysis
- Basic documentation

### Acceptance Criteria

- Application launches.
- Test suite runs.
- Build succeeds.
- Architecture is documented.
- Domain code can execute without rendering/UI dependencies.
- No unnecessary future-feature implementation.

### Review Checkpoint

STOP.

Report:

- Technology choices
- Project structure
- Dependency graph
- How state is owned
- Test setup
- Known limitations

Do not begin Phase 2.

---

## Phase 2 — Music Theory & Chord Engine

### Goal

Build a reliable, UI-independent music engine.

### Implement

- Notes
- Pitch classes
- Enharmonic spelling
- Keys
- Chord parser
- Chord model
- Transposition
- Slash chords
- Extensions
- Alterations
- Suspensions
- Unit tests

### Acceptance Criteria

The engine can correctly parse and transpose the supported chord families.

Examples must include:

```text
C → D
Cm → Dm
F# → G#
Bb → C
G/B → A/C#
F#m7b5/C# → G#m7b5/D#
```

The test suite must contain both valid and invalid cases.

### Review Checkpoint

STOP.

Do not build UI around the engine until the engine is reviewable.

---

## Phase 3 — Canonical Song Model

### Goal

Build the normalized representation used throughout the application.

### Implement

- Song
- Metadata
- Sections
- Lines
- Chord tokens
- Lyric tokens
- Annotations
- Source metadata
- Serialization/deserialization

### Acceptance Criteria

A song can be represented without relying on raw source text.

Chord and lyric positions remain recoverable.

### Review Checkpoint

STOP.

---

## Phase 4 — ChordPro & Plain Text Import

### Goal

Allow users to turn real-world chord sheets into canonical songs.

### Implement

- ChordPro parser
- Plain text chord-over-lyrics parser
- Error reporting
- Unknown chord preservation
- Import fixtures
- Import tests

### Acceptance Criteria

Imported content is editable and renderable through the canonical model.

Malformed input does not destroy usable content.

### Review Checkpoint

STOP.

---

## Phase 5 — Song Viewer

### Goal

Create the first genuinely usable song-reading experience.

### Implement

- Song viewer
- Chord/lyric alignment
- Section headers
- Responsive layout
- Font scaling
- Light/dark themes
- Basic transpose controls

### Acceptance Criteria

A user can:

1. Import a song.
2. View it.
3. Change its key.
4. Read chords and lyrics without losing alignment.

### Review Checkpoint

STOP.

---

## Phase 6 — Library & Persistence

### Goal

Turn the viewer into a usable songbook.

### Implement

- Local persistence
- Song library
- Search
- Favorites
- Tags
- Metadata editing
- Delete
- Duplicate
- Recent songs

### Acceptance Criteria

Songs survive application restarts and remain available offline.

### Review Checkpoint

STOP.

---

## Phase 7 — Editing & Manual Song Creation

### Goal

Allow users to create and correct songs without external software.

### Implement

- New Song
- Song editor
- Chord tagging
- Section creation
- Metadata editing
- Parser correction workflows
- Save/cancel behavior

### Acceptance Criteria

A user can create a complete chord chart from scratch and reopen it later.

### Review Checkpoint

STOP.

---

## Phase 8 — Setlists

### Goal

Support organized rehearsal and performance.

### Implement

- Setlist model
- Setlist UI
- Add/remove songs
- Reorder songs
- Setlist-specific key
- Setlist-specific capo
- Setlist notes
- Duplicate setlist

### Acceptance Criteria

A setlist can contain the same song multiple times without duplicating the underlying song document.

Each entry can have independent performance settings.

### Review Checkpoint

STOP.

---

## Phase 9 — Live Performance Mode

### Goal

Make the application practical for real performances.

### Implement

- Performance mode
- Fullscreen
- Large typography
- Dark stage mode
- Next/previous song
- Setlist progress
- Auto-scroll
- Scroll-speed control
- Keep-awake behavior where supported
- Touch/gesture navigation where practical
- Desktop keyboard navigation

### Acceptance Criteria

A musician can perform an entire set without needing to return to the editor.

### Review Checkpoint

STOP.

---

## Phase 10 — MusicXML & Sheet Music

### Goal

Expand beyond text-based chord charts.

### Implement

- MusicXML import
- MXL import
- Score model
- Sheet music rendering
- Appropriate notation library integration
- MusicXML-specific error handling

### Acceptance Criteria

Supported MusicXML documents render correctly and remain usable offline.

### Review Checkpoint

STOP.

---

## Phase 11 — Accessibility, Polish & Reliability

### Goal

Make the application production-quality.

### Implement

- Accessibility review
- Keyboard navigation
- Screen-reader labels
- High-contrast behavior
- Touch target review
- Performance optimization
- Loading states
- Empty states
- Error states
- Data-loss safeguards
- Regression tests
- UI consistency pass

### Acceptance Criteria

No known critical workflows require unreliable behavior or unnecessary interaction.

### Review Checkpoint

STOP.

---

## Phase 12 — Packaging & Release

### Goal

Prepare the application for real-world distribution.

### Implement

- Production builds
- Platform packaging
- Versioning
- App metadata
- Release configuration
- Installation/update strategy
- Production documentation
- Final automated test run

### Acceptance Criteria

A clean environment can install and launch the application successfully.

### Review Checkpoint

STOP.

---

# 24. Definition of Done

A feature is not complete merely because it works in the happy path.

A phase is complete only when:

- Requirements are implemented.
- Automated tests exist for important behavior.
- Existing tests still pass.
- No obvious regressions exist.
- The application builds successfully.
- The application can be launched.
- Error paths have been considered.
- Data is not silently lost.
- Architecture remains consistent.
- No duplicated business logic was introduced.
- Documentation is updated where architectural behavior changed.
- The phase acceptance criteria are satisfied.

---

# 25. Agent Development Rules

These rules apply to every phase.

## 25.1 Phase Discipline

- Implement only the requested phase.
- Do not silently implement later phases.
- Do not create speculative infrastructure that complicates the current phase.
- Stop at the review checkpoint.

## 25.2 Architecture

- Keep domain logic UI-independent.
- Do not duplicate state ownership.
- Do not duplicate music-theory logic.
- Do not parse raw song text inside UI components.
- Do not make the renderer authoritative over song data.
- Do not make setlists duplicate entire song documents.

## 25.3 Data Safety

- Never silently discard user content.
- Preserve unknown input whenever possible.
- Do not overwrite source data merely because normalized data changed.
- Avoid destructive migrations without safeguards.

## 25.4 Testing

- Write tests alongside implementation.
- Do not delete or weaken tests to make a feature pass.
- Add regression tests for significant bugs.
- Prefer deterministic tests.
- Keep music-theory tests exhaustive and explicit.

## 25.5 Dependencies

Before adding a dependency:

1. Determine whether the existing project already provides the functionality.
2. Determine whether the dependency is actively maintained and appropriate.
3. Determine whether it works on every required platform.
4. Explain why it is necessary.
5. Avoid dependencies for trivial functionality.

## 25.6 Error Handling

Never hide exceptions or failures simply to make the UI appear functional.

Errors must either:

- be handled meaningfully,
- be surfaced to the user,
- or be propagated to an appropriate higher-level handler.

## 25.7 Assumptions

When a requirement is ambiguous:

1. Choose the simplest behavior consistent with the product.
2. Document the assumption.
3. Implement it consistently.
4. Do not invent large new features to resolve minor ambiguity.

---

# 26. UX Principles

The application should follow these principles:

### Fast

Common actions should require minimal interaction.

### Readable

The song is more important than the application chrome.

### Predictable

Changing the key should never unexpectedly alter lyrics, structure, or unrelated metadata.

### Forgiving

Bad imports and unusual chords should degrade gracefully.

### Offline-first

A musician should never discover during a performance that an internet connection was required.

### Performance-first

Live mode should eliminate unnecessary controls and distractions.

### Consistent

The same song model and music engine must drive import, editing, rendering, transposition, and performance mode.

---

# 27. Optional / Future Features

These are deliberately not MVP requirements.

Potential future additions include:

## Performance

- Metronome
- Tap tempo
- Count-in
- Custom page-turner mappings
- Bluetooth pedal support
- Automatic scrolling based on tempo
- Performance annotations

## Musical Features

- Chord diagrams
- Guitar fingering suggestions
- Alternate tunings
- Nashville number system
- Roman numeral analysis
- Scale/key suggestions
- Instrument-specific transposition
- Instrument transposition presets
- Custom chord voicings

## Organization

- Advanced tagging
- Smart setlists
- Templates
- Rehearsal notes
- Song ratings
- Practice tracking

## Import/Export

- PDF import
- Additional chord-chart formats
- Export to ChordPro
- Export to PDF
- Print-friendly layouts

## Cloud

- Optional synchronization
- Backup
- Multi-device libraries
- Account-based sharing

Cloud features must not compromise the local-first architecture.

---

# 28. Example End-to-End Scenario

A musician imports a ChordPro file:

```text
{title: Example Song}
{key: G}

[G]Amazing grace, how [D]sweet the sound
That [Em]saved a wretch like [C]me
```

The application normalizes it into a Song.

The musician sets:

```text
Original key: G
Performance key: Bb
```

The displayed chords become:

```text
[Bb]Amazing grace, how [F]sweet the sound
That [Gm]saved a wretch like [Eb]me
```

The musician saves the song.

They create:

```text
Sunday Morning Set
```

and add the song.

For this particular performance, they override the key:

```text
Performance key: A
```

During the performance they enter Live Mode.

They see:

```text
Sunday Morning Set
Song 1 / 8

Amazing grace, how sweet the sound
That saved a wretch like me
```

with large readable chords and lyrics.

They enable auto-scroll.

At any point they can transpose again without leaving performance mode.

The underlying song remains intact, and the setlist-specific arrangement remains independent from the song's default arrangement.

---

# 29. Final Product Requirement

The finished application should satisfy the following overarching statement:

> A musician must be able to take a song from an external chord source, import it, correct or edit it, transpose it accurately into another key, save it locally, place it into a setlist, and perform that setlist entirely offline with minimal interaction and highly readable presentation.

Every architectural and product decision should support that workflow.

---

# 30. Final Agent Instruction

Treat this document as the current source of truth.

Before beginning any implementation:

1. Inspect the repository.
2. Identify existing code and constraints.
3. Compare the repository against the current phase requirements.
4. Implement only the current phase.
5. Add or update tests.
6. Run the relevant tests.
7. Verify the application builds/runs.
8. Review the implementation against the acceptance criteria.
9. Report what changed, what was tested, and any known limitations.
10. **STOP.**

Do not continue into the next phase until explicitly instructed.

The goal is not merely to produce a functioning demo. The goal is to build a maintainable, testable, music-theory-accurate application whose architecture can support the later phases without requiring fundamental rewrites.
