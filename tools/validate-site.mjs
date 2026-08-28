import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve("site");
const files = ["index.html", "privacy/index.html", "code-signing/index.html", "styles.css"];
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
requireText("index.html", ">Code signing policy<", "SITE_SIGNING_POLICY_LINK_MISSING");
requireText("code-signing/index.html", "Free code signing provided by", "SITE_SIGNPATH_ATTRIBUTION_MISSING");
requireText("code-signing/index.html", "Committer and reviewer", "SITE_SIGNING_ROLE_MISSING");
requireText("code-signing/index.html", "Signing approver", "SITE_APPROVER_ROLE_MISSING");
requireText("code-signing/index.html", "Every production release requires manual approval", "SITE_MANUAL_APPROVAL_MISSING");
requireText("code-signing/index.html", "This program will not transfer any information", "SITE_SIGNING_PRIVACY_STATEMENT_MISSING");

for (const [file, source] of Object.entries(contents)) {
  if (/<script\b/i.test(source)) errors.push(`SITE_SCRIPT_FORBIDDEN: ${file}`);
  if (/<(?:script|img|iframe|link)\b[^>]*\b(?:src|href)=["']https?:\/\/(?!tigramaan\.github\.io)/i.test(source)) {
    errors.push(`SITE_EXTERNAL_RUNTIME_ASSET_FORBIDDEN: ${file}`);
  }
  if (/<(?:form|iframe)\b/i.test(source)) errors.push(`SITE_INTERACTIVE_EMBED_FORBIDDEN: ${file}`);
}

if (errors.length > 0) {
  console.error(errors.join("\n"));
  process.exit(1);
}

console.log(JSON.stringify({ ok: true, files, runtimeScripts: 0, analytics: 0, forms: 0 }));
