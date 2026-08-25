/**
 * Declares Android storage permissions so Documents/Tonic can work after the
 * user grants “All files access” (sideload-friendly).
 */
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(new URL(".", import.meta.url)), "..");
const manifestPath = join(
  root,
  "src-tauri",
  "gen",
  "android",
  "app",
  "src",
  "main",
  "AndroidManifest.xml",
);

if (!existsSync(manifestPath)) {
  console.log("skip apply-android-storage: AndroidManifest.xml not found");
  process.exit(0);
}

let manifest = readFileSync(manifestPath, "utf8");
let changed = false;

const permissionBlock = [
  '    <uses-permission android:name="android.permission.MANAGE_EXTERNAL_STORAGE" />',
  '    <uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" android:maxSdkVersion="32" />',
  '    <uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="29" />',
].join("\n");

if (!manifest.includes("MANAGE_EXTERNAL_STORAGE")) {
  if (!manifest.includes("<application")) {
    console.error("AndroidManifest.xml missing <application>");
    process.exit(1);
  }
  manifest = manifest.replace(
    /(\s*)(<application\b)/,
    `\n${permissionBlock}\n$1$2`,
  );
  changed = true;
}

if (!/requestLegacyExternalStorage\s*=\s*"true"/.test(manifest)) {
  const next = manifest.replace(
    /<application\b([^>]*?)(\/?)>/,
    (full, attrs, selfClose) => {
      if (/requestLegacyExternalStorage/.test(attrs)) {
        return full;
      }
      return `<application${attrs} android:requestLegacyExternalStorage="true"${selfClose}>`;
    },
  );
  if (next !== manifest) {
    manifest = next;
    changed = true;
  }
}

if (changed) {
  writeFileSync(manifestPath, manifest, "utf8");
  console.log(`Patched storage permissions → ${manifestPath}`);
} else {
  console.log("Android storage permissions already present");
}
