//! Library folder resolution: user-visible on desktop; private app-data on Android.
//!
//! Android shared paths like Documents/Tonic need storage permissions that sideloaded
//! APKs do not have by default. Preferring them caused an immediate SIGABRT on startup
//! when setup returned an error (see logcat: MediaProvider permission denied + tonic_lib abort).

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_opener::OpenerExt;
#[cfg(not(target_os = "android"))]
use tonic_persist::{copy_library_tree, library_has_data};
use tonic_persist::{FileLibrary, SongLibrary};

const BACKUP_README: &str = "\
Tonic song library

This folder is the live library. Copy the whole Tonic folder to Google Drive to back up.
Paste a backup here (or drop JSON files into songs/) and return to Tonic — the app
rereads this folder when you come back.

- songs/     each chart as a JSON file
- setlists/  setlist JSON files
- index.json library counters

Desktop: Documents → Tonic
Android: app-private storage (use Open save folder in Settings); shared Documents is not used without storage permission.
";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenLibraryFolderResult {
    pub path: String,
    pub opened: bool,
    pub message: String,
}

/// Resolve a writable library root. Never prefers an unreadable Android shared folder.
pub fn resolve_library_root<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let legacy = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("library");

    // Android: private app data only. Shared Documents/Download needs permissions we
    // do not request yet; using them aborts Tauri setup when open fails.
    #[cfg(target_os = "android")]
    {
        FileLibrary::open(&legacy).map_err(|error| error.to_string())?;
        let _ = write_backup_readme(&legacy);
        return Ok(legacy);
    }

    #[cfg(not(target_os = "android"))]
    {
        let mut candidates = visible_roots(app);
        if !candidates.iter().any(|path| path == &legacy) {
            candidates.push(legacy.clone());
        }

        for path in &candidates {
            if library_has_data(path) {
                match FileLibrary::open(path) {
                    Ok(_) => {
                        let _ = write_backup_readme(path);
                        return Ok(path.clone());
                    }
                    Err(_) => continue,
                }
            }
        }

        for path in candidates.iter().filter(|path| *path != &legacy) {
            if probe_writable_library(path) {
                if library_has_data(&legacy) {
                    if let Err(error) = copy_library_tree(&legacy, path) {
                        eprintln!("library migration skipped: {error}");
                    }
                }
                let _ = write_backup_readme(path);
                return Ok(path.clone());
            }
        }

        FileLibrary::open(&legacy).map_err(|error| error.to_string())?;
        let _ = write_backup_readme(&legacy);
        Ok(legacy)
    }
}

/// Open the live save folder in the system file manager.
pub fn open_save_folder<R: Runtime>(
    app: &AppHandle<R>,
    live_root: &Path,
) -> Result<OpenLibraryFolderResult, String> {
    let _ = write_backup_readme(live_root);
    let opened = reveal_or_open(app, live_root);
    let path = live_root.display().to_string();
    let message = if opened {
        format!("Opened the save folder:\n{path}")
    } else {
        format!(
            "This is the live song folder:\n{path}\n\nOn desktop, look under Documents → Tonic. On Android the library is in app-private storage — use a file manager that can open app data, or copy files via USB after enabling access."
        )
    };
    Ok(OpenLibraryFolderResult {
        path,
        opened,
        message,
    })
}

#[cfg(not(target_os = "android"))]
fn visible_roots<R: Runtime>(app: &AppHandle<R>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(docs) = app.path().document_dir() {
        roots.push(docs.join("Tonic"));
    }
    roots
}

#[cfg(not(target_os = "android"))]
fn probe_writable_library(path: &Path) -> bool {
    if FileLibrary::open(path)
        .and_then(|library| library.health_check())
        .is_err()
    {
        return false;
    }
    let probe = path.join(".tonic_write_probe");
    let ok = fs::write(&probe, b"ok").is_ok() && fs::remove_file(&probe).is_ok();
    ok
}

fn write_backup_readme(root: &Path) -> std::io::Result<()> {
    fs::write(root.join("HOW_TO_BACKUP.txt"), BACKUP_README)
}

fn reveal_or_open<R: Runtime>(app: &AppHandle<R>, path: &Path) -> bool {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        app.opener().reveal_item_in_dir(path).is_ok()
            || app
                .opener()
                .open_path(path.to_string_lossy(), None::<&str>)
                .is_ok()
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        if let Some(uri) = android_documents_uri(path) {
            if app.opener().open_url(uri, None::<&str>).is_ok() {
                return true;
            }
        }
        app.opener()
            .open_path(path.to_string_lossy(), None::<&str>)
            .is_ok()
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn android_documents_uri(path: &Path) -> Option<String> {
    let text = path.to_string_lossy().replace('\\', "/");
    let relative = if let Some(index) = text.find("/Documents/Tonic") {
        &text[index + 1..]
    } else if let Some(index) = text.find("/Download/Tonic") {
        &text[index + 1..]
    } else if let Some(index) = text.find("/Downloads/Tonic") {
        &text[index + 1..]
    } else {
        return None;
    };
    let encoded = relative.replace('/', "%2F");
    Some(format!(
        "content://com.android.externalstorage.documents/document/primary%3A{encoded}"
    ))
}
