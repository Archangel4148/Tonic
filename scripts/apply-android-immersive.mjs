/**
 * Copies immersive MainActivity into the generated Android project.
 * Safe to re-run after `tauri android init`.
 */
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
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

const packagePath = identifier.split(".").join("/");
const targetPath = join(
  androidRoot,
  "app",
  "src",
  "main",
  "java",
  packagePath,
  "MainActivity.kt",
);

mkdirSync(dirname(targetPath), { recursive: true });

let source = readFileSync(templatePath, "utf8");
source = source.replace(/^package\s+[\w.]+/m, `package ${identifier}`);

writeFileSync(targetPath, source, "utf8");
console.log(`Applied immersive MainActivity → ${targetPath}`);
