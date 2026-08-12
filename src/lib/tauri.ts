import { invoke } from "@tauri-apps/api/core";
import type { AppInfo } from "./types";

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}
