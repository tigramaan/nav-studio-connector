import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import process from "node:process";
import { chromium } from "playwright-core";

const viewports = [
  [320, 568],
  [390, 844],
  [768, 1024],
  [1024, 768],
  [1440, 900],
  [1920, 1080],
  [2560, 1440],
  [3840, 2160],
  [1440, 1600]
];

const browserCandidates = [
  process.env.CONNECTOR_QA_BROWSER,
  "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
  "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
  "/usr/bin/microsoft-edge",
  "/usr/bin/google-chrome",
  "/usr/bin/chromium-browser",
  "/usr/bin/chromium"
].filter(Boolean);

const executablePath = browserCandidates.find((candidate) => existsSync(candidate));
if (!executablePath) {
  console.error("VISUAL_QA_BROWSER_MISSING: set CONNECTOR_QA_BROWSER to Chromium/Edge");
  process.exit(4);
}

const targetUrl = process.argv[2] ?? "http://127.0.0.1:1420/?demo=1";
const outputDirectory = resolve(process.argv[3] ?? "artifacts/ui-qa-playwright");
mkdirSync(outputDirectory, { recursive: true });

const browser = await chromium.launch({ executablePath, headless: true });
const report = [];
let failed = false;

try {
  for (const [width, height] of viewports) {
    const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: 1 });
    const page = await context.newPage();
    const consoleErrors = [];
    const failedRequests = [];
    page.on("console", (message) => {
      if (message.type() === "error") consoleErrors.push(message.text());
    });
    page.on("requestfailed", (request) => failedRequests.push({ url: request.url(), error: request.failure()?.errorText ?? "unknown" }));

    await page.goto(targetUrl, { waitUntil: "networkidle", timeout: 15_000 });
    await page.emulateMedia({ reducedMotion: "reduce" });
    const metrics = await page.evaluate(() => {
      const viewportWidth = window.innerWidth;
      const overflowElements = [...document.querySelectorAll("body *")]
        .map((element) => {
          const rect = element.getBoundingClientRect();
          return { tag: element.tagName.toLowerCase(), className: element.className, left: rect.left, right: rect.right, width: rect.width };
        })
        .filter((entry) => entry.width > 0 && (entry.left < -1 || entry.right > viewportWidth + 1))
        .slice(0, 10);
      const undersizedTargets = [...document.querySelectorAll("button, input, a[href]")]
        .filter((element) => {
          const rect = element.getBoundingClientRect();
          return rect.width > 0 && rect.height > 0 && (rect.width < 44 || rect.height < 44);
        })
        .map((element) => {
          const rect = element.getBoundingClientRect();
          return { tag: element.tagName.toLowerCase(), className: element.className, width: rect.width, height: rect.height };
        });
      return {
        innerWidth: viewportWidth,
        scrollWidth: document.documentElement.scrollWidth,
        bodyScrollWidth: document.body.scrollWidth,
        overflowElements,
        undersizedTargets
      };
    });

    const name = `${width}x${height}`;
    await page.screenshot({ path: resolve(outputDirectory, `${name}-viewport.png`), fullPage: false });
    await page.screenshot({ path: resolve(outputDirectory, `${name}-full.png`), fullPage: true });
    const viewportPassed = metrics.scrollWidth <= metrics.innerWidth && metrics.bodyScrollWidth <= metrics.innerWidth;
    const passed = viewportPassed && metrics.undersizedTargets.length === 0 && consoleErrors.length === 0 && failedRequests.length === 0;
    if (!passed) failed = true;
    report.push({ viewport: name, passed, metrics, consoleErrors, failedRequests });
    await context.close();
  }
} finally {
  await browser.close();
}

writeFileSync(resolve(outputDirectory, "report.json"), `${JSON.stringify(report, null, 2)}\n`, "utf8");
console.log(JSON.stringify({ ok: !failed, browser: executablePath, targetUrl, outputDirectory, viewports: report }, null, 2));
process.exit(failed ? 1 : 0);
