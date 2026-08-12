# Phase 12 — Packaging & Release

**Status:** Implemented, pending review  
**Do not start further work beyond this phase unless explicitly instructed.**

## Goal

Prepare Tonic for real-world distribution with the simplest possible install path: one desktop installer on a blank PC, one Android APK for sideload.

## What shipped

- App version **1.0.0** aligned across `package.json`, workspace `Cargo.toml`, and `src-tauri/tauri.conf.json`
- Product metadata (category, descriptions, publisher, homepage)
- Platform-specific bundle targets:
  - Windows: **NSIS only**, `webviewInstallMode: offlineInstaller`, `installMode: currentUser`
  - macOS: **DMG only**
  - Linux: **AppImage only**
- npm scripts: `package:desktop`, `package:desktop:windows`, `android:init`, `package:android`, `release:check`
- GitHub Actions: desktop multi-OS release + Android APK workflow
- `.gitignore` allows committing `src-tauri/gen/android` while ignoring other generated Tauri output and keystores
- Docs: [`docs/release.md`](../release.md); this report; README / development / architecture updates
- Product phase reported by `AppServices` is **12**

## Acceptance criteria

| Criterion                                               | Result                                                                                                                                                                                                            |
| ------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A clean environment can install and launch successfully | Windows: `npm run package:desktop:windows` produced `Tonic_1.0.0_x64-setup.exe` (NSIS + offline WebView2, per-user). Android: single sideload APK via `android:init` + build (SDK/CI; not built on this machine). |

## Review notes

- Auto-update is intentionally **not** included; updates are reinstall / replace APK (see release docs).
- Android project files are not in the repo until someone with the SDK runs `npm run android:init` (or CI does). Packaging config and scripts are ready.
- This machine did not have `ANDROID_HOME`; local APK generation was not run here. Use CI or install Android Studio to produce the APK.

## Known limitations

- First Android build requires SDK/NDK and `android:init`
- Release APK signing keystore is operator-managed (not committed)
- macOS/Linux artifacts need macOS/Linux builders (CI matrix)

## How to review

```bash
npm run release:check
npm run package:desktop:windows   # on Windows — produces NSIS under target/release/bundle/nsis
```

Read [`docs/release.md`](../release.md). On a machine with Android SDK: `npm run android:init` then `npm run package:android`. Optionally run the GitHub Actions workflows and confirm one installer + one APK artifact.
