import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve("site");
const files = ["index.html", "privacy/index.html", "styles.css"];
const contents = Object.fromEntries(files.map((file) => [file, readFileSync(resolve(root, file), "utf8")]));
const errors = [];

function requireText(file, value, code) {
  if (!contents[file].includes(value)) errors.push(`${code}: ${file} is missing ${value}`);
}

requireText("index.html", "https://github.com/tigramaan/nav-studio-connector/releases", "SITE_RELEASE_LINK_MISSING");
requireText("index.html", "privacy/", "SITE_PRIVACY_LINK_MISSING");
requireText("index.html", ">MIT License<", "SITE_LICENSE_LINK_MISSING");
requireText("privacy/index.html", "uses no analytics, advertising, cookies, forms, or external runtime assets", "SITE_PRIVACY_DISCLOSURE_MISSING");
requireText("privacy/index.html", "does not collect application telemetry", "SITE_TELEMETRY_DISCLOSURE_MISSING");

for (const [file, source] of Object.entries(contents)) {
  if (/<script\b/i.test(source)) errors.push(`SITE_SCRIPT_FORBIDDEN: ${file}`);
  if (/\b(?:src|href)=["']https?:\/\/(?!github\.com|tigramaan\.github\.io)/i.test(source)) {
    errors.push(`SITE_EXTERNAL_RUNTIME_ASSET_FORBIDDEN: ${file}`);
  }
  if (/<(?:form|iframe)\b/i.test(source)) errors.push(`SITE_INTERACTIVE_EMBED_FORBIDDEN: ${file}`);
}

if (errors.length > 0) {
  console.error(errors.join("\n"));
  process.exit(1);
}

console.log(JSON.stringify({ ok: true, files, runtimeScripts: 0, analytics: 0, forms: 0 }));
