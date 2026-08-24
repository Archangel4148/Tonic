/**
 * Copies immersive MainActivity into the generated Android project.
 * Safe to re-run after `tauri android init`.
 *
 * Finds the existing MainActivity.kt (do not invent a second package path).
 */
import {
  existsSync,
  readdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(new URL(".", import.meta.url)), "..");
const templatePath = join(root, "src-tauri", "android", "MainActivity.kt");
const confPath = join(root, "src-tauri", "tauri.conf.json");
const androidRoot = join(root, "src-tauri", "gen", "android");

if (!existsSync(androidRoot)) {
  console.log("skip apply-android-immersive: src-tauri/gen/android not found");
  process.exit(0);
}

if (!existsSync(templatePath)) {
  console.error(`Missing template: ${templatePath}`);
  process.exit(1);
}

const conf = JSON.parse(readFileSync(confPath, "utf8"));
const identifier = conf.identifier;
if (typeof identifier !== "string" || !identifier.includes(".")) {
  console.error(`Invalid app identifier in tauri.conf.json: ${identifier}`);
  process.exit(1);
}

function findMainActivity(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "build" || entry.name === ".gradle") {
        continue;
      }
      const found = findMainActivity(path);
      if (found) {
        return found;
      }
    } else if (entry.name === "MainActivity.kt") {
      return path;
    }
  }
  return null;
}

const targetPath = findMainActivity(androidRoot);
if (!targetPath) {
  console.error(
    "Could not find MainActivity.kt under src-tauri/gen/android. Run android:init first.",
  );
  process.exit(1);
}

let source = readFileSync(templatePath, "utf8");
source = source.replace(/^package\s+[\w.`]+/m, `package ${identifier}`);

writeFileSync(targetPath, source, "utf8");
console.log(`Applied immersive MainActivity → ${targetPath}`);
