/**
 * Declares Android storage permissions and a deep link so “Use Documents folder”
 * can open the All-files settings screen from MainActivity (no Rust JNI).
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

// Ensure MainActivity can receive tonic://request-all-files while running.
if (!/android:launchMode\s*=\s*"singleTop"/.test(manifest)) {
  const next = manifest.replace(
    /<activity\b([^>]*android:name\s*=\s*"\.MainActivity"[^>]*)>/,
    (full, attrs) => {
      if (/android:launchMode/.test(attrs)) {
        return full;
      }
      return `<activity${attrs} android:launchMode="singleTop">`;
    },
  );
  if (next !== manifest) {
    manifest = next;
    changed = true;
  }
}

if (
  !manifest.includes("tonic://request-all-files") &&
  !manifest.includes('android:host="request-all-files"')
) {
  const filter = `
            <intent-filter>
                <action android:name="android.intent.action.VIEW" />
                <category android:name="android.intent.category.DEFAULT" />
                <category android:name="android.intent.category.BROWSABLE" />
                <data android:scheme="tonic" android:host="request-all-files" />
            </intent-filter>`;
  // Insert before the closing </activity> that belongs to MainActivity when possible.
  const mainActivityClose = manifest.search(
    /android:name\s*=\s*"\.MainActivity"[\s\S]*?<\/activity>/,
  );
  if (mainActivityClose >= 0) {
    const closeIdx = manifest.indexOf("</activity>", mainActivityClose);
    if (closeIdx >= 0) {
      manifest =
        manifest.slice(0, closeIdx) +
        filter +
        "\n        " +
        manifest.slice(closeIdx);
      changed = true;
    }
  } else {
    // Fallback: first </activity>
    const closeIdx = manifest.indexOf("</activity>");
    if (closeIdx >= 0) {
      manifest =
        manifest.slice(0, closeIdx) +
        filter +
        "\n        " +
        manifest.slice(closeIdx);
      changed = true;
    }
  }
}

if (changed) {
  writeFileSync(manifestPath, manifest, "utf8");
  console.log(`Patched storage permissions / deep link → ${manifestPath}`);
} else {
  console.log("Android storage permissions already present");
}
