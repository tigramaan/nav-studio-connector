import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const license = await readFile(resolve(root, "LICENSE"), "utf8");
const required = [
  "MIT License",
  "Copyright (c) 2026 tigramaan",
  "Permission is hereby granted, free of charge",
  'THE SOFTWARE IS PROVIDED "AS IS"'
];
const missing = required.filter((value) => !license.includes(value));
const result = { ok: missing.length === 0, license: "MIT", attribution: "tigramaan", missing };
console.log(JSON.stringify(result));
if (!result.ok) process.exit(1);
