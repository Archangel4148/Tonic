/**
 * Sync app version across package.json, package-lock.json, Cargo.toml, tauri.conf.json.
 *
 * Usage:
 *   node scripts/set-version.mjs 1.2.3
 *   node scripts/set-version.mjs v1.2.3
 */
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(new URL(".", import.meta.url)), "..");
const raw = process.argv[2];
if (!raw) {
  console.error("Usage: node scripts/set-version.mjs <version>");
  process.exit(1);
}

const version = String(raw).trim().replace(/^v/i, "");
if (!/^\d+\.\d+\.\d+([.-][0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`Invalid semver-ish version: ${raw}`);
  process.exit(1);
}

function writeJson(path, mutate) {
  const data = JSON.parse(readFileSync(path, "utf8"));
  mutate(data);
  writeFileSync(path, `${JSON.stringify(data, null, 2)}\n`, "utf8");
}

writeJson(join(root, "package.json"), (pkg) => {
  pkg.version = version;
});

writeJson(join(root, "package-lock.json"), (lock) => {
  lock.version = version;
  if (lock.packages?.[""]) {
    lock.packages[""].version = version;
  }
});

writeJson(join(root, "src-tauri", "tauri.conf.json"), (conf) => {
  conf.version = version;
});

const cargoPath = join(root, "Cargo.toml");
const cargo = readFileSync(cargoPath, "utf8");
// First bare `version = "..."` line is [workspace.package] (deps use inline tables).
const nextCargo = cargo.replace(
  /^version\s*=\s*"[^"]+"/m,
  `version = "${version}"`,
);
if (nextCargo === cargo) {
  console.error("Could not update [workspace.package] version in Cargo.toml");
  process.exit(1);
}
writeFileSync(cargoPath, nextCargo, "utf8");

console.log(`Set version to ${version}`);
