// Fails unless a release tag is vMAJOR.MINOR.PATCH and every version manifest says the
// same MAJOR.MINOR.PATCH. Usage: node scripts/check-release-version.mjs v0.1.0
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8");

const tag = process.argv[2] ?? "";
const match = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.exec(tag);
if (!match) {
  console.error(`Tag '${tag}' is not vMAJOR.MINOR.PATCH (for example v0.1.0).`);
  process.exit(1);
}
const version = tag.slice(1);

const packageLock = JSON.parse(read("package-lock.json"));
const cargoToml = /^\[package\][^[]*?^version\s*=\s*"([^"]+)"/m.exec(read("src-tauri/Cargo.toml"));
const cargoLock = /^\[\[package\]\]\r?\nname = "bob"\r?\nversion = "([^"]+)"/m.exec(read("src-tauri/Cargo.lock"));

const found = {
  "package.json": JSON.parse(read("package.json")).version,
  "package-lock.json": packageLock.version,
  'package-lock.json packages[""]': packageLock.packages?.[""]?.version,
  "src-tauri/tauri.conf.json": JSON.parse(read("src-tauri/tauri.conf.json")).version,
  "src-tauri/Cargo.toml": cargoToml?.[1],
  "src-tauri/Cargo.lock (bob)": cargoLock?.[1],
};

const mismatched = Object.entries(found).filter(([, value]) => value !== version);
for (const [file, value] of Object.entries(found)) {
  console.log(`${value === version ? "ok " : "BAD"} ${file}: ${value ?? "(not found)"}`);
}
if (mismatched.length > 0) {
  console.error(`Tag ${tag} does not match ${mismatched.map(([file]) => file).join(", ")}.`);
  process.exit(1);
}
console.log(`All version manifests match ${tag}.`);
