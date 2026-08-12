# Development

## Prerequisites

- Node.js 24+ (npm is fine)
- Rust stable (`rustc` 1.77+; Phase 1 was started on 1.97)
- Tauri 2 desktop prerequisites for your OS:
  - Windows: WebView2 and the MSVC C++ build tools
  - macOS: Xcode command line tools
  - Linux: WebKitGTK and the packages listed in the Tauri 2 desktop guide

Android SDK/NDK are **not** required for Phase 1.

## Install

```bash
npm install
```

Rust crate dependencies are fetched automatically by Cargo.

## Run

Desktop app (required for Rust IPC):

```bash
npm run tauri dev
```

Frontend only, in a browser:

```bash
npm run dev
```

Browser-only mode cannot call the Rust engine. The shell will show an engine-unavailable message. Import, library, setlists, transpose, and live mode need `npm run tauri dev`. Fullscreen is best in the desktop window.

## Test, lint, format

```bash
npm test
npm run lint
npm run format
npm run check
```

See [testing.md](./testing.md) for details.

## Build

Frontend production assets:

```bash
npm run build
```

Native desktop bundle:

```bash
npm run tauri build
```

Android is out of scope until packaging work begins. When that happens, the starting command will be `npm run tauri android init`, then `npm run tauri android dev`.

## Editor

Recommended VS Code / Cursor extensions are listed in `.vscode/extensions.json`:

- Tauri
- rust-analyzer (linked to the workspace `Cargo.toml`)
- ESLint
- Prettier

## Phase discipline

Implement only the requested phase. Keep the app buildable and runnable after every phase. Stop at the review checkpoint and wait for an explicit instruction before starting the next phase.
