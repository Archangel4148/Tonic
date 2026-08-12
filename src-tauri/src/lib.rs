//! Tauri shell for Tonic.
//!
//! This crate is an IPC and windowing boundary. It must not contain music-theory
//! algorithms or become the owner of song data.

use serde::Serialize;
use tonic_app::{
    performance_key_choices, AppServices, EditorMetaUpdate, EditorSaveResult, EditorSessionView,
    ImportMode, LibraryListView, LibraryQuery, MetadataUpdate, SectionLabelInput,
    SetlistMetaUpdate, SetlistSummaryView, SetlistView, SongSessionView,
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
fn editor_add_section(
    services: tauri::State<'_, AppServices>,
    label: SectionLabelInput,
) -> Result<EditorSessionView, String> {
    services.editor_add_section(label)
}

#[tauri::command]
fn editor_set_section_label(
    services: tauri::State<'_, AppServices>,
    index: usize,
    label: SectionLabelInput,
) -> Result<EditorSessionView, String> {
    services.editor_set_section_label(index, label)
}

#[tauri::command]
fn editor_remove_section(
    services: tauri::State<'_, AppServices>,
    index: usize,
) -> Result<EditorSessionView, String> {
    services.editor_remove_section(index)
}

#[tauri::command]
fn editor_move_section(
    services: tauri::State<'_, AppServices>,
    from: usize,
    to: usize,
) -> Result<EditorSessionView, String> {
    services.editor_move_section(from, to)
}

#[tauri::command]
fn editor_add_line(
    services: tauri::State<'_, AppServices>,
    section: usize,
) -> Result<EditorSessionView, String> {
    services.editor_add_line(section)
}

#[tauri::command]
fn editor_remove_line(
    services: tauri::State<'_, AppServices>,
    section: usize,
    line: usize,
) -> Result<EditorSessionView, String> {
    services.editor_remove_line(section, line)
}

#[tauri::command]
fn editor_set_lyrics(
    services: tauri::State<'_, AppServices>,
    section: usize,
    line: usize,
    lyrics: String,
) -> Result<EditorSessionView, String> {
    services.editor_set_lyrics(section, line, lyrics)
}

#[tauri::command]
fn editor_tag_chord(
    services: tauri::State<'_, AppServices>,
    section: usize,
    line: usize,
    lyric_index: u32,
    symbol: String,
) -> Result<EditorSessionView, String> {
    services.editor_tag_chord(section, line, lyric_index, symbol)
}

#[tauri::command]
fn editor_untag_chord(
    services: tauri::State<'_, AppServices>,
    section: usize,
    line: usize,
    chord_index: usize,
) -> Result<EditorSessionView, String> {
    services.editor_untag_chord(section, line, chord_index)
}

#[tauri::command]
fn editor_replace_chord(
    services: tauri::State<'_, AppServices>,
    section: usize,
    line: usize,
    chord_index: usize,
    symbol: String,
) -> Result<EditorSessionView, String> {
    services.editor_replace_chord(section, line, chord_index, symbol)
}

#[tauri::command]
fn editor_set_chord_index(
    services: tauri::State<'_, AppServices>,
    section: usize,
    line: usize,
    chord_index: usize,
    lyric_index: u32,
) -> Result<EditorSessionView, String> {
    services.editor_set_chord_index(section, line, chord_index, lyric_index)
}

#[tauri::command]
fn editor_set_annotation(
    services: tauri::State<'_, AppServices>,
    section: usize,
    line: usize,
    text: Option<String>,
) -> Result<EditorSessionView, String> {
    services.editor_set_annotation(section, line, text)
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

#[tauri::command]
fn editor_parse_body(
    services: tauri::State<'_, AppServices>,
    text: String,
    format: Option<String>,
) -> Result<EditorSessionView, String> {
    let mode = ImportMode::parse(format.as_deref().unwrap_or("auto"))?;
    services.editor_parse_body(&text, mode)
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
            import_song,
            import_binary,
            import_url,
            current_song,
            transpose_song,
            set_performance_key,
            reset_performance_key,
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
            editor_add_section,
            editor_set_section_label,
            editor_remove_section,
            editor_move_section,
            editor_add_line,
            editor_remove_line,
            editor_set_lyrics,
            editor_tag_chord,
            editor_untag_chord,
            editor_replace_chord,
            editor_set_chord_index,
            editor_set_annotation,
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
