import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const spec = await readFile(resolve(root, "specs/001-desktop-connector/spec.md"), "utf8");
const matrix = await readFile(resolve(root, "specs/TRACEABILITY_MATRIX.md"), "utf8");
const requirements = [...new Set(spec.match(/REQ-\d{3}/g) ?? [])].sort();
const missing = requirements.filter((id) => !matrix.includes(id));
const result = { ok: missing.length === 0, requirements: requirements.length, missing };
console.log(JSON.stringify(result));
if (!result.ok) process.exit(1);
