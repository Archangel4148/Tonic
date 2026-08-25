/**
 * Copies immersive MainActivity into the generated Android project and ensures
 * androidx.core is on the app classpath (needed for WindowInsetsControllerCompat).
 *
 * Safe to re-run after `tauri android init`.
 * Finds the existing MainActivity.kt (do not invent a second package path).
 */
import {
  existsSync,
  readdirSync,
  readFileSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(new URL(".", import.meta.url)), "..");
const templatePath = join(root, "src-tauri", "android", "MainActivity.kt");
const confPath = join(root, "src-tauri", "tauri.conf.json");
const androidRoot = join(root, "src-tauri", "gen", "android");
const gradlePath = join(androidRoot, "app", "build.gradle.kts");

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

function findFiles(dir, name, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "build" || entry.name === ".gradle") {
        continue;
      }
      findFiles(path, name, out);
    } else if (entry.name === name) {
      out.push(path);
    }
  }
  return out;
}

const targets = findFiles(androidRoot, "MainActivity.kt");
if (targets.length === 0) {
  console.error(
    "Could not find MainActivity.kt under src-tauri/gen/android. Run android:init first.",
  );
  process.exit(1);
}

let source = readFileSync(templatePath, "utf8");
source = source.replace(/^package\s+[\w.`]+/m, `package ${identifier}`);

// Prefer the path that already matches the identifier package layout.
const preferred =
  targets.find((path) =>
    path.replace(/\\/g, "/").includes(`/${identifier.split(".").join("/")}/`),
  ) ?? targets[0];

for (const path of targets) {
  if (path === preferred) {
    writeFileSync(path, source, "utf8");
    console.log(`Applied immersive MainActivity → ${path}`);
  } else {
    unlinkSync(path);
    console.log(`Removed duplicate MainActivity → ${path}`);
  }
}

if (existsSync(gradlePath)) {
  let gradle = readFileSync(gradlePath, "utf8");
  const dep = 'implementation("androidx.core:core-ktx:1.13.1")';
  if (!gradle.includes("androidx.core:core-ktx")) {
    if (/dependencies\s*\{/.test(gradle)) {
      gradle = gradle.replace(
        /dependencies\s*\{/,
        `dependencies {\n    ${dep}`,
      );
      writeFileSync(gradlePath, gradle, "utf8");
      console.log(`Added ${dep} to app/build.gradle.kts`);
    } else {
      console.warn(
        "Could not patch app/build.gradle.kts for androidx.core — immersive may crash if core-ktx is missing.",
      );
    }
  }
}
