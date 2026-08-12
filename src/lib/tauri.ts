import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  ImportFormat,
  LibraryList,
  LibraryQuery,
  MetadataUpdate,
  SongSession,
} from "./types";

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}

export async function importSong(
  text: string,
  format: ImportFormat = "auto",
): Promise<SongSession> {
  return invoke<SongSession>("import_song", { text, format });
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

export async function toggleFavorite(
  id: string,
): Promise<SongSession | null> {
  return invoke<SongSession | null>("library_toggle_favorite", { id });
}

export async function updateMetadata(
  update: MetadataUpdate,
): Promise<SongSession> {
  return invoke<SongSession>("library_update_metadata", { update });
}
