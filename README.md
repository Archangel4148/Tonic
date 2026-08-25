# Tonic

A local-first musician’s songbook for importing, editing, transposing, and performing chord charts — offline, on your own device.

Works on **Windows**, **macOS**, **Linux**, and **Android** (sideloaded APK). No account, no cloud, no app store required.

## Download

Get the latest installers and APK from the GitHub Releases page:

**→ [Latest release](https://github.com/Archangel4148/Tonic/releases/latest)**

| Platform | File to download                         | How to install                                                                 |
| -------- | ---------------------------------------- | ------------------------------------------------------------------------------ |
| Windows  | `Tonic_*_x64-setup.exe`                  | Double-click. No admin needed. Works on PCs without Edge/WebView2.             |
| macOS    | `Tonic_*_*.dmg`                          | Open the DMG and drag Tonic to Applications.                                   |
| Linux    | `Tonic_*_*.AppImage`                     | Make it executable (`chmod +x …`) and run it.                                  |
| Android  | `Tonic_*_universal.apk`                  | Allow “Install unknown apps,” then open the APK.                               |

Updating: download the new build for the same platform and install over the previous one. Your song library stays on the device (Documents/Tonic on desktop; app storage or Documents/Tonic on Android).

More detail: [`docs/release.md`](./docs/release.md).

## Features (short)

- Import ChordPro, plain text, Ultimate Guitar URLs, and MusicXML
- Edit charts, transpose by key or capo, organize setlists
- Live/stage mode with auto-scroll and setlist navigation
- Everything stored locally — you own the files

## For developers

```bash
npm install
npm run tauri dev
```

See [`docs/development.md`](./docs/development.md) for setup, tests, and packaging. Product and engineering docs live under [`docs/`](./docs/README.md).

## License

MIT
