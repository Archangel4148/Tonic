//! Library folder resolution.
//!
//! Desktop: prefer Documents/Tonic (user-visible).
//! Android: prefer Documents/Tonic when “All files access” is granted; otherwise use
//! Android/data/<package>/files/Tonic (no permission, visible over USB / many file managers);
//! fall back to private app-data library/.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_opener::OpenerExt;
use tonic_persist::{copy_library_tree, library_has_data, FileLibrary, SongLibrary};

const BACKUP_README: &str = "\
Tonic song library

This folder is the live library. Copy the whole Tonic folder to Google Drive to back up.
Paste a backup here (or drop JSON files into songs/) and return to Tonic — the app
rereads this folder when you come back.

- songs/     each chart as a JSON file
- setlists/  setlist JSON files
- index.json library counters

Desktop: Documents → Tonic
Android (with All files access): Documents → Tonic
Android (default): Android/data/com.tonic.songbook/files/Tonic (USB / file manager)
";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenLibraryFolderResult {
    pub path: String,
    pub opened: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStorageStatus {
    pub library_path: Option<String>,
    pub kind: String,
    pub documents_path: Option<String>,
    pub documents_writable: bool,
    pub has_all_files_access: bool,
    pub can_use_documents: bool,
    pub hint: String,
}

/// Resolve a writable library root without aborting on unreadable shared folders.
pub fn resolve_library_root<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let legacy = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("library");

    #[cfg(target_os = "android")]
    {
        return resolve_android(app, legacy);
    }

    #[cfg(not(target_os = "android"))]
    {
        resolve_desktop(app, legacy)
    }
}

pub fn storage_status<R: Runtime>(
    app: &AppHandle<R>,
    live_root: Option<&Path>,
) -> LibraryStorageStatus {
    #[cfg(target_os = "android")]
    {
        let documents = android_documents_tonic(app);
        let has_access = crate::android_storage::has_all_files_access();
        let documents_writable = documents
            .as_ref()
            .is_some_and(|path| probe_writable_library(path));
        let kind = classify_android_path(live_root);
        let hint = if documents_writable {
            "Library can use Documents/Tonic (visible in the Files app). Restart Tonic after granting access if the location has not switched yet.".to_string()
        } else if has_access {
            "All files access is on, but Documents/Tonic is not writable yet. Try Rescan or restart.".to_string()
        } else {
            "Default location is Android/data/…/files/Tonic (USB-friendly). Tap “Use Documents folder” to grant All files access for Files-app / Drive backups.".to_string()
        };
        LibraryStorageStatus {
            library_path: live_root.map(|path| path.display().to_string()),
            kind,
            documents_path: documents.map(|path| path.display().to_string()),
            documents_writable,
            has_all_files_access: has_access,
            can_use_documents: documents_writable,
            hint,
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        LibraryStorageStatus {
            library_path: live_root.map(|path| path.display().to_string()),
            kind: "documents".to_string(),
            documents_path: live_root.map(|path| path.display().to_string()),
            documents_writable: true,
            has_all_files_access: true,
            can_use_documents: true,
            hint: "Desktop libraries live under Documents/Tonic.".to_string(),
        }
    }
}

pub fn request_documents_access() -> Result<LibraryStorageStatus, String> {
    #[cfg(target_os = "android")]
    {
        crate::android_storage::request_all_files_access()?;
        Ok(LibraryStorageStatus {
            library_path: None,
            kind: "pending".to_string(),
            documents_path: None,
            documents_writable: false,
            has_all_files_access: crate::android_storage::has_all_files_access(),
            can_use_documents: false,
            hint: "Grant “Allow access to manage all files” for Tonic, then return here and restart the app (or tap Rescan after relaunch).".to_string(),
        })
    }
    #[cfg(not(target_os = "android"))]
    {
        Err("Documents access is only required on Android.".to_string())
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
            "This is the live song folder:\n{path}\n\nDesktop: Documents → Tonic.\nAndroid: Files app → Documents → Tonic (if All files access is on), or Android/data/com.tonic.songbook/files/Tonic."
        )
    };
    Ok(OpenLibraryFolderResult {
        path,
        opened,
        message,
    })
}

#[cfg(target_os = "android")]
fn resolve_android<R: Runtime>(app: &AppHandle<R>, legacy: PathBuf) -> Result<PathBuf, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(documents) = android_documents_tonic(app) {
        // Only try Documents when permission/probe allows — never crash on denial.
        if crate::android_storage::has_all_files_access() || probe_writable_library(&documents) {
            candidates.push(documents);
        }
    }
    if let Some(external) = android_external_files_tonic(app) {
        candidates.push(external);
    }
    candidates.push(legacy.clone());

    for path in &candidates {
        if library_has_data(path) && probe_writable_library(path) {
            let _ = write_backup_readme(path);
            return Ok(path.clone());
        }
    }

    for path in &candidates {
        if probe_writable_library(path) {
            if path != &legacy && library_has_data(&legacy) {
                if let Err(error) = copy_library_tree(&legacy, path) {
                    eprintln!("library migration skipped: {error}");
                }
            }
            // Prefer migrating from external → Documents when Documents becomes available.
            if let Some(external) = android_external_files_tonic(app) {
                if path != &external && library_has_data(&external) && path != &legacy {
                    if let Err(error) = copy_library_tree(&external, path) {
                        eprintln!("library migration skipped: {error}");
                    }
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

#[cfg(not(target_os = "android"))]
fn resolve_desktop<R: Runtime>(app: &AppHandle<R>, legacy: PathBuf) -> Result<PathBuf, String> {
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

#[cfg(target_os = "android")]
fn android_documents_tonic<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path()
        .home_dir()
        .ok()
        .map(|home| home.join("Documents").join("Tonic"))
}

#[cfg(target_os = "android")]
fn android_external_files_tonic<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    let id = app.config().identifier.clone();
    app.path().home_dir().ok().map(|home| {
        home.join("Android")
            .join("data")
            .join(id)
            .join("files")
            .join("Tonic")
    })
}

#[cfg(target_os = "android")]
fn classify_android_path(live_root: Option<&Path>) -> String {
    let Some(path) = live_root else {
        return "unknown".to_string();
    };
    let text = path.to_string_lossy().replace('\\', "/");
    if text.contains("/Documents/Tonic") {
        "documents".to_string()
    } else if text.contains("/Android/data/") {
        "appExternal".to_string()
    } else {
        "appPrivate".to_string()
    }
}

#[cfg(not(target_os = "android"))]
fn visible_roots<R: Runtime>(app: &AppHandle<R>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(docs) = app.path().document_dir() {
        roots.push(docs.join("Tonic"));
    }
    roots
}

fn probe_writable_library(path: &Path) -> bool {
    if FileLibrary::open(path)
        .and_then(|library| library.health_check())
        .is_err()
    {
        return false;
    }
    let probe = path.join(".tonic_write_probe");
    fs::write(&probe, b"ok").is_ok() && fs::remove_file(&probe).is_ok()
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
    } else if let Some(index) = text.find("/Android/data/") {
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
