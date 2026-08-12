# Release & installation

Tonic ships as **one desktop installer per OS** and **one Android APK**. No app store, no auto-updater server — install or replace the build you download.

## What end users install

| Platform | Artifact                           | How to install                                                                                                                                           |
| -------- | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows  | `Tonic_1.0.0_x64-setup.exe` (NSIS) | Double-click; no admin required (per-user install). Bundles the full WebView2 offline installer so a blank PC without Edge/WebView2 still works offline. |
| macOS    | `Tonic_1.0.0_*.dmg`                | Open the DMG and drag Tonic to Applications.                                                                                                             |
| Linux    | `Tonic_1.0.0_*.AppImage`           | Make executable and run (`chmod +x … && ./…`).                                                                                                           |
| Android  | `Tonic-*.apk` (universal)          | Enable “Install unknown apps” for your file manager/browser, then open the APK.                                                                          |

Version numbers in filenames follow the release version in `package.json` / `src-tauri/tauri.conf.json` / workspace `Cargo.toml` (currently **1.0.0**).

## Updates

1. Download the new installer or APK for the same platform.
2. Install over the previous build (Windows/macOS) or install the new APK (Android may ask to update/replace).
3. Library data stays in the app data directory; do not delete app data when upgrading.

There is no in-app auto-update. Reinstall is the update strategy.

## Build desktop (developers)

Prerequisites: Node 24+, Rust stable, [Tauri 2 desktop deps](https://v2.tauri.app/start/prerequisites/).

```bash
npm install
npm run release:check   # format, lint, tests, frontend build
npm run package:desktop # OS-specific single target (see platform configs)
```

On Windows you can force NSIS only:

```bash
npm run package:desktop:windows
```

Outputs land under `src-tauri/target/release/bundle/` (e.g. `nsis/`, `dmg/`, `appimage/`).

Platform merge configs:

- `src-tauri/tauri.windows.conf.json` — NSIS + offline WebView2 + current-user install
- `src-tauri/tauri.macos.conf.json` — DMG only
- `src-tauri/tauri.linux.conf.json` — AppImage only

## Build Android APK (developers)

### One-time SDK setup

1. Install [Android Studio](https://developer.android.com/studio) (or command-line tools).
2. Install SDK Platform 34+, Build-Tools, NDK, and Android SDK Command-line Tools.
3. Set environment variables (adjust paths for your machine):

```powershell
# Windows PowerShell example
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
$env:NDK_HOME = "$env:ANDROID_HOME\ndk\<version>"
```

4. Add Rust Android targets:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

5. Generate the Android project (commit `src-tauri/gen/android` once it exists):

```bash
npm run android:init
```

### Package the APK

```bash
npm run package:android
```

Look under `src-tauri/gen/android/app/build/outputs/apk/` for a **universal** APK. That single file is what you sideload.

For a quick unsigned/debug installable build during development:

```bash
npx tauri android build --debug --apk
```

Release signing: configure a keystore in the generated Android project (or CI secrets). Never commit `.jks` / `.keystore` files.

## CI

GitHub Actions workflows under `.github/workflows/`:

- `release-desktop.yml` — builds Windows NSIS, macOS DMG, Linux AppImage on tag `v*` or manual dispatch
- `release-android.yml` — initializes Android (if needed), builds a universal APK artifact

Upload artifacts from the Actions run, or attach them to a GitHub Release.

## Version bump checklist

1. Set the same version in `package.json`, workspace `Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Run `npm install` so `package-lock.json` matches.
3. Run `npm run release:check`.
4. Tag `vX.Y.Z` and let CI produce installers/APK (or build locally).
5. Publish the **one** installer/APK per platform you support for that release.
