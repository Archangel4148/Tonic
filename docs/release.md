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
3. Library data lives in **Documents/Tonic** (or the app data `library/` folder on older installs). Do not delete those files when upgrading.

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

`android:init` also runs `npm run icons` and `npm run android:immersive` afterward. That matters because `tauri android init` seeds the default Tauri launcher icon into `gen/android`; `tauri icon` only writes into that project once it exists. Without the icon step, Settings can show the correct web icon while the home-screen launcher stays the Tauri template. The immersive step patches `MainActivity` so the status and navigation bars stay hidden until you swipe from the edge.

### Android signing (one-time setup)

**Why this matters:** Android only allows in-place APK updates when the new build has the **same package id** (`com.tonic.songbook`) and the **same signing certificate**. Debug builds from CI, local dev, or different machines each get their own debug certificate, so Android reports errors like “App not installed”, “incompatible package”, or similar — and you must uninstall first (which wipes app data, including your song library).

Use **one release keystore** for every sideload build you distribute.

1. Create a keystore locally (once). Remember the passwords and alias:

```powershell
keytool -genkey -v -keystore tonic-release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias upload
```

2. Base64-encode it for GitHub Actions secrets:

```powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes("tonic-release.jks"))
```

3. In the GitHub repo, add **Settings → Secrets and variables → Actions**:

| Secret | Value |
| ------ | ----- |
| `ANDROID_KEY_BASE64` | Base64 output from step 2 |
| `ANDROID_KEY_PASSWORD` | Keystore password |
| `ANDROID_KEY_ALIAS` | `upload` (or the alias you chose) |

4. **Back up `tonic-release.jks` somewhere safe.** If you lose it, future APKs cannot upgrade existing installs; users would have to uninstall (losing local library data unless exported).

The CI workflow builds a **release** APK and signs it with this keystore. Every release from that workflow can update the previous one without re-importing songs.

**One-time migration:** If you already installed an older debug CI APK, uninstall it once, then install the first release-signed build. After that, future CI APKs should upgrade in place and keep your library.

### Package the APK

```bash
npm run package:android
```

Look under `src-tauri/gen/android/app/build/outputs/apk/` for a **universal** APK. That single file is what you sideload.

For a quick unsigned/debug build during local development only:

```bash
npx tauri android build --debug --apk
```

Do not distribute debug APKs to users; they will not upgrade each other reliably.

## CI

GitHub Actions workflows under `.github/workflows/`:

- `release-desktop.yml` — builds Windows NSIS, macOS DMG, Linux AppImage on tag `v*` or manual dispatch
- `build-android-debug.yml` — **manual only**: universal **debug** APK, no keystore or version tag needed (for dev testing on a phone)
- `release-android.yml` — tag `v*` or manual: universal **release** APK signed with the upload keystore from GitHub secrets (for distribution / in-place updates)

Upload artifacts from the Actions run, or attach release artifacts to a GitHub Release.

### Which Android workflow to use

| Goal | Workflow | Notes |
| ---- | -------- | ----- |
| Try the app on your phone while developing | **Build Android APK (debug)** | Run manually from Actions. Uninstall old build first if install fails. Debug builds do not upgrade each other reliably. |
| Ship a version users can update in place | **Release Android APK** | Tag `vX.Y.Z` (bump version first) or manual dispatch after signing secrets are configured. |

Do not mix debug and release installs when testing updates — they use different signing keys. Pick one track per device until you move to release signing for good.

## Version bump checklist

1. Set the same version in `package.json`, workspace `Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Run `npm install` so `package-lock.json` matches.
3. Run `npm run release:check`.
4. Tag `vX.Y.Z` and let CI produce installers/APK (or build locally).
5. Publish the **one** installer/APK per platform you support for that release.
