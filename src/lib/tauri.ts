import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  EditorMetaUpdate,
  EditorSaveResult,
  EditorSession,
  ImportFormat,
  LibraryInfo,
  LibraryList,
  LibraryQuery,
  LibraryReloadResult,
  MetadataUpdate,
  OpenLibraryFolderResult,
  LibraryStorageStatus,
  Setlist,
  SetlistMetaUpdate,
  SetlistSummary,
  SongSession,
} from "./types";

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}

export async function getLibraryInfo(): Promise<LibraryInfo> {
  return invoke<LibraryInfo>("library_info");
}

export async function openLibraryFolder(): Promise<OpenLibraryFolderResult> {
  return invoke<OpenLibraryFolderResult>("library_open_folder");
}

export async function getLibraryStorageStatus(): Promise<LibraryStorageStatus> {
  return invoke<LibraryStorageStatus>("library_storage_status");
}

export async function requestDocumentsAccess(): Promise<LibraryStorageStatus> {
  return invoke<LibraryStorageStatus>("library_request_documents_access");
}

export async function reloadLibrary(): Promise<LibraryReloadResult> {
  return invoke<LibraryReloadResult>("library_reload");
}

export async function clearLibrary(): Promise<void> {
  return invoke<void>("library_clear");
}

export async function importSong(
  text: string,
  format: ImportFormat = "auto",
): Promise<SongSession> {
  return invoke<SongSession>("import_song", { text, format });
}

export async function importBinary(
  bytes: Uint8Array,
  fileName?: string,
): Promise<SongSession> {
  return invoke<SongSession>("import_binary", {
    bytes: Array.from(bytes),
    fileName: fileName ?? null,
  });
}

export async function importUrl(url: string): Promise<SongSession> {
  return invoke<SongSession>("import_url", { url });
}

export async function getCurrentSong(): Promise<SongSession | null> {
  return invoke<SongSession | null>("current_song");
}

export async function transposeSong(semitones: number): Promise<SongSession> {
  return invoke<SongSession>("transpose_song", { semitones });
}

export async function setPerformanceKey(key: string): Promise<SongSession> {
  return invoke<SongSession>("set_performance_key", { key });
}

export async function resetPerformanceKey(): Promise<SongSession> {
  return invoke<SongSession>("reset_performance_key");
}

export async function setTransposeMode(
  mode: "chords" | "capo",
): Promise<SongSession> {
  return invoke<SongSession>("set_transpose_mode", { mode });
}

export async function setShapesKey(key: string): Promise<SongSession> {
  return invoke<SongSession>("set_shapes_key", { key });
}

export async function clearSong(): Promise<void> {
  return invoke<void>("clear_song");
}

export async function listLibrary(
  query: LibraryQuery = {},
): Promise<LibraryList> {
  return invoke<LibraryList>("library_list", { query });
}

export async function openLibrarySong(id: string): Promise<SongSession> {
  return invoke<SongSession>("library_open", { id });
}

export async function deleteLibrarySong(
  id: string,
): Promise<SongSession | null> {
  return invoke<SongSession | null>("library_delete", { id });
}

export async function duplicateLibrarySong(id: string): Promise<SongSession> {
  return invoke<SongSession>("library_duplicate", { id });
}

export async function toggleFavorite(id: string): Promise<SongSession | null> {
  return invoke<SongSession | null>("library_toggle_favorite", { id });
}

export async function updateMetadata(
  update: MetadataUpdate,
): Promise<SongSession> {
  return invoke<SongSession>("library_update_metadata", { update });
}

export async function createSong(): Promise<EditorSession> {
  return invoke<EditorSession>("editor_create");
}

export async function beginEdit(id: string): Promise<EditorSession> {
  return invoke<EditorSession>("editor_begin", { id });
}

export async function getEditorState(): Promise<EditorSession | null> {
  return invoke<EditorSession | null>("editor_state");
}

export async function saveEdit(): Promise<EditorSaveResult> {
  return invoke<EditorSaveResult>("editor_save");
}

export async function cancelEdit(): Promise<SongSession | null> {
  return invoke<SongSession | null>("editor_cancel");
}

export async function editorUpdateMeta(
  update: EditorMetaUpdate,
): Promise<EditorSession> {
  return invoke<EditorSession>("editor_update_meta", { update });
}

export async function editorParseBody(text: string): Promise<EditorSession> {
  return invoke<EditorSession>("editor_parse_body", { text });
}

export async function listSetlists(): Promise<SetlistSummary[]> {
  return invoke<SetlistSummary[]>("setlist_list");
}

export async function getSetlist(id: string): Promise<Setlist> {
  return invoke<Setlist>("setlist_get", { id });
}

export async function createSetlist(name?: string | null): Promise<Setlist> {
  return invoke<Setlist>("setlist_create", { name: name ?? null });
}

export async function updateSetlistMeta(
  id: string,
  update: SetlistMetaUpdate,
): Promise<Setlist> {
  return invoke<Setlist>("setlist_update_meta", { id, update });
}

export async function deleteSetlist(id: string): Promise<void> {
  return invoke<void>("setlist_delete", { id });
}

export async function duplicateSetlist(id: string): Promise<Setlist> {
  return invoke<Setlist>("setlist_duplicate", { id });
}

export async function addSetlistSong(
  setlistId: string,
  songId: string,
): Promise<Setlist> {
  return invoke<Setlist>("setlist_add_song", { setlistId, songId });
}

export async function removeSetlistEntry(
  setlistId: string,
  entryId: string,
): Promise<Setlist> {
  return invoke<Setlist>("setlist_remove_entry", { setlistId, entryId });
}

export async function moveSetlistEntry(
  setlistId: string,
  from: number,
  to: number,
): Promise<Setlist> {
  return invoke<Setlist>("setlist_move_entry", { setlistId, from, to });
}

export async function updateSetlistEntry(
  setlistId: string,
  entryId: string,
  performanceKey: string | null,
  capoFret: number | null,
  notes: string | null,
): Promise<Setlist> {
  return invoke<Setlist>("setlist_update_entry", {
    setlistId,
    entryId,
    performanceKey,
    capoFret,
    notes,
  });
}

export async function openSetlistEntry(
  setlistId: string,
  entryId: string,
): Promise<SongSession> {
  return invoke<SongSession>("setlist_open_entry", { setlistId, entryId });
}

export async function openSetlistNeighbor(delta: number): Promise<SongSession> {
  return invoke<SongSession>("setlist_open_neighbor", { delta });
}
