//! Tauri shell for Tonic.
//!
//! This crate is an IPC and windowing boundary. It must not contain music-theory
//! algorithms or become the owner of song data.

use serde::Serialize;
use tonic_app::AppServices;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfoResponse {
    name: String,
    version: String,
    phase: u32,
    domain_engine: String,
    domain_version: String,
    persistence_healthy: bool,
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
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppServices::new())
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
