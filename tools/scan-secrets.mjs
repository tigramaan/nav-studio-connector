import { readdir, readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, extname, relative, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const ignored = new Set([".git", "node_modules", "target", "dist", "artifacts"]);
const binaryExtensions = new Set([".png", ".ico", ".icns", ".exe", ".dll", ".so", ".zip", ".deb", ".appimage"]);
const patterns = [
  /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/,
  /\bAKIA[0-9A-Z]{16}\b/,
  /\bsk-(?:proj-)?[A-Za-z0-9_-]{20,}\b/,
  /(?:password|passwd|access[_-]?token|private[_-]?key)\s*[:=]\s*["'][^"']{8,}["']/i
];
const findings = [];

async function walk(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (ignored.has(entry.name)) continue;
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) { await walk(path); continue; }
    if (binaryExtensions.has(extname(entry.name).toLowerCase())) continue;
    const text = await readFile(path, "utf8").catch(() => "");
    if (patterns.some((pattern) => pattern.test(text))) findings.push(relative(root, path));
  }
}

await walk(root);
console.log(JSON.stringify({ ok: findings.length === 0, findings }));
if (findings.length) process.exit(1);
