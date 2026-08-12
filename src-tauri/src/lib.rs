//! Tauri shell for Tonic.
//!
//! This crate is an IPC and windowing boundary. It must not contain music-theory
//! algorithms or become the owner of song data.

use serde::Serialize;
use tonic_app::{performance_key_choices, AppServices, ImportMode, SongSessionView};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfoResponse {
    name: String,
    version: String,
    phase: u32,
    domain_engine: String,
    domain_version: String,
    persistence_healthy: bool,
    performance_keys: Vec<String>,
}

#[tauri::command]
fn app_info(services: tauri::State<'_, AppServices>) -> AppInfoResponse {
    let info = services.info();
    AppInfoResponse {
        name: info.name.to_string(),
        version: info.version.to_string(),
        phase: info.phase,
        domain_engine: info.domain_engine.to_string(),
        domain_version: info.domain_version.to_string(),
        persistence_healthy: services.persistence_healthy(),
        performance_keys: performance_key_choices(),
    }
}

#[tauri::command]
fn import_song(
    services: tauri::State<'_, AppServices>,
    text: String,
    format: Option<String>,
) -> Result<SongSessionView, String> {
    let mode = ImportMode::parse(format.as_deref().unwrap_or("auto"))?;
    Ok(services.import_text(&text, mode))
}

#[tauri::command]
fn current_song(services: tauri::State<'_, AppServices>) -> Option<SongSessionView> {
    services.current_session()
}

#[tauri::command]
fn transpose_song(
    services: tauri::State<'_, AppServices>,
    semitones: i32,
) -> Result<SongSessionView, String> {
    services.transpose_by(semitones)
}

#[tauri::command]
fn set_performance_key(
    services: tauri::State<'_, AppServices>,
    key: String,
) -> Result<SongSessionView, String> {
    services.set_performance_key(&key)
}

#[tauri::command]
fn reset_performance_key(
    services: tauri::State<'_, AppServices>,
) -> Result<SongSessionView, String> {
    services.reset_performance_key()
}

#[tauri::command]
fn clear_song(services: tauri::State<'_, AppServices>) {
    services.clear_song();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppServices::new())
        .invoke_handler(tauri::generate_handler![
            app_info,
            import_song,
            current_song,
            transpose_song,
            set_performance_key,
            reset_performance_key,
            clear_song,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
