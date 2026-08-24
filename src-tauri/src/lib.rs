//! Tauri shell for Tonic.
//!
//! This crate is an IPC and windowing boundary. It must not contain music-theory
//! algorithms or become the owner of song data.

use serde::Serialize;
use tonic_app::{
    performance_key_choices, AppServices, EditorMetaUpdate, EditorSaveResult, EditorSessionView,
    ImportMode, LibraryInfoView, LibraryListView, LibraryQuery, MetadataUpdate, SetlistMetaUpdate,
    SetlistSummaryView, SetlistView, SongSessionView,
};

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
fn library_info(services: tauri::State<'_, AppServices>) -> LibraryInfoView {
    services.library_info()
}

#[tauri::command]
fn library_clear(services: tauri::State<'_, AppServices>) -> Result<(), String> {
    services.clear_library()
}

#[tauri::command]
fn import_song(
    services: tauri::State<'_, AppServices>,
    text: String,
    format: Option<String>,
) -> Result<SongSessionView, String> {
    let mode = ImportMode::parse(format.as_deref().unwrap_or("auto"))?;
    services.import_text(&text, mode)
}

#[tauri::command]
fn import_binary(
    services: tauri::State<'_, AppServices>,
    bytes: Vec<u8>,
    file_name: Option<String>,
) -> Result<SongSessionView, String> {
    services.import_bytes(&bytes, file_name.as_deref())
}

#[tauri::command]
fn import_url(
    services: tauri::State<'_, AppServices>,
    url: String,
) -> Result<SongSessionView, String> {
    services.import_url(&url)
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
fn set_transpose_mode(
    services: tauri::State<'_, AppServices>,
    mode: String,
) -> Result<SongSessionView, String> {
    services.set_transpose_mode(&mode)
}

#[tauri::command]
fn clear_song(services: tauri::State<'_, AppServices>) -> Result<(), String> {
    services.close_song()
}

#[tauri::command]
fn library_list(
    services: tauri::State<'_, AppServices>,
    query: Option<LibraryQuery>,
) -> LibraryListView {
    services.list_library(query.unwrap_or_default())
}

#[tauri::command]
fn library_open(
    services: tauri::State<'_, AppServices>,
    id: String,
) -> Result<SongSessionView, String> {
    services.open_song(&id)
}

#[tauri::command]
fn library_delete(
    services: tauri::State<'_, AppServices>,
    id: String,
) -> Result<Option<SongSessionView>, String> {
    services.delete_song(&id)
}

#[tauri::command]
fn library_duplicate(
    services: tauri::State<'_, AppServices>,
    id: String,
) -> Result<SongSessionView, String> {
    services.duplicate_song(&id)
}

#[tauri::command]
fn library_toggle_favorite(
    services: tauri::State<'_, AppServices>,
    id: String,
) -> Result<Option<SongSessionView>, String> {
    services.toggle_favorite(&id)
}

#[tauri::command]
fn library_update_metadata(
    services: tauri::State<'_, AppServices>,
    update: MetadataUpdate,
) -> Result<SongSessionView, String> {
    services.update_open_metadata(update)
}

#[tauri::command]
fn editor_create(services: tauri::State<'_, AppServices>) -> Result<EditorSessionView, String> {
    services.create_song()
}

#[tauri::command]
fn editor_begin(
    services: tauri::State<'_, AppServices>,
    id: String,
) -> Result<EditorSessionView, String> {
    services.begin_edit(&id)
}

#[tauri::command]
fn editor_state(services: tauri::State<'_, AppServices>) -> Option<EditorSessionView> {
    services.editor_state()
}

#[tauri::command]
fn editor_save(services: tauri::State<'_, AppServices>) -> Result<EditorSaveResult, String> {
    services.save_edit()
}

#[tauri::command]
fn editor_cancel(services: tauri::State<'_, AppServices>) -> Option<SongSessionView> {
    services.cancel_edit()
}

#[tauri::command]
fn editor_update_meta(
    services: tauri::State<'_, AppServices>,
    update: EditorMetaUpdate,
) -> Result<EditorSessionView, String> {
    services.editor_update_meta(update)
}

#[tauri::command]
fn editor_parse_body(
    services: tauri::State<'_, AppServices>,
    text: String,
) -> Result<EditorSessionView, String> {
    services.editor_parse_body(&text)
}

#[tauri::command]
fn setlist_list(services: tauri::State<'_, AppServices>) -> Vec<SetlistSummaryView> {
    services.list_setlists()
}

#[tauri::command]
fn setlist_get(services: tauri::State<'_, AppServices>, id: String) -> Result<SetlistView, String> {
    services.get_setlist(&id)
}

#[tauri::command]
fn setlist_create(
    services: tauri::State<'_, AppServices>,
    name: Option<String>,
) -> Result<SetlistView, String> {
    services.create_setlist(name)
}

#[tauri::command]
fn setlist_update_meta(
    services: tauri::State<'_, AppServices>,
    id: String,
    update: SetlistMetaUpdate,
) -> Result<SetlistView, String> {
    services.update_setlist_meta(&id, update)
}

#[tauri::command]
fn setlist_delete(services: tauri::State<'_, AppServices>, id: String) -> Result<(), String> {
    services.delete_setlist(&id)
}

#[tauri::command]
fn setlist_duplicate(
    services: tauri::State<'_, AppServices>,
    id: String,
) -> Result<SetlistView, String> {
    services.duplicate_setlist(&id)
}

#[tauri::command]
fn setlist_add_song(
    services: tauri::State<'_, AppServices>,
    setlist_id: String,
    song_id: String,
) -> Result<SetlistView, String> {
    services.add_setlist_song(&setlist_id, &song_id)
}

#[tauri::command]
fn setlist_remove_entry(
    services: tauri::State<'_, AppServices>,
    setlist_id: String,
    entry_id: String,
) -> Result<SetlistView, String> {
    services.remove_setlist_entry(&setlist_id, &entry_id)
}

#[tauri::command]
fn setlist_move_entry(
    services: tauri::State<'_, AppServices>,
    setlist_id: String,
    from: usize,
    to: usize,
) -> Result<SetlistView, String> {
    services.move_setlist_entry(&setlist_id, from, to)
}

#[tauri::command]
fn setlist_update_entry(
    services: tauri::State<'_, AppServices>,
    setlist_id: String,
    entry_id: String,
    performance_key: Option<String>,
    capo_fret: Option<u8>,
    notes: Option<String>,
) -> Result<SetlistView, String> {
    services.update_setlist_entry(&setlist_id, &entry_id, performance_key, capo_fret, notes)
}

#[tauri::command]
fn setlist_open_entry(
    services: tauri::State<'_, AppServices>,
    setlist_id: String,
    entry_id: String,
) -> Result<SongSessionView, String> {
    services.open_setlist_entry(&setlist_id, &entry_id)
}

#[tauri::command]
fn setlist_open_neighbor(
    services: tauri::State<'_, AppServices>,
    delta: i32,
) -> Result<SongSessionView, String> {
    services.open_setlist_neighbor(delta)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            let root = app.path().app_data_dir()?.join("library");
            app.manage(AppServices::open(&root)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            library_info,
            library_clear,
            import_song,
            import_binary,
            import_url,
            current_song,
            transpose_song,
            set_performance_key,
            reset_performance_key,
            set_transpose_mode,
            clear_song,
            library_list,
            library_open,
            library_delete,
            library_duplicate,
            library_toggle_favorite,
            library_update_metadata,
            editor_create,
            editor_begin,
            editor_state,
            editor_save,
            editor_cancel,
            editor_update_meta,
            editor_parse_body,
            setlist_list,
            setlist_get,
            setlist_create,
            setlist_update_meta,
            setlist_delete,
            setlist_duplicate,
            setlist_add_song,
            setlist_remove_entry,
            setlist_move_entry,
            setlist_update_entry,
            setlist_open_entry,
            setlist_open_neighbor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
