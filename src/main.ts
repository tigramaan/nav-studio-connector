import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

type Robot = {
  robot_id: string;
  display_name: string;
  hostname: string;
  addresses: string[];
  port: number;
  studio_url: string;
  model?: string;
  api_version?: string;
};

type Inspection = {
  studio_url: string;
  hostname: string;
  resolved_address: string;
  fingerprint_sha256: string;
  subject: string;
  issuer: string;
  not_before: string;
  not_after: string;
  latency_ms: number;
};

type ConnectorError = {
  code?: string;
  message?: string;
  retryable?: boolean;
};

const state: {
  phase: "idle" | "discovering" | "select" | "inspecting" | "verify" | "installing" | "ready" | "error";
  robots: Robot[];
  selected?: Robot;
  inspection?: Inspection;
  error?: ConnectorError;
  diagnostics?: unknown;
} = { phase: "idle", robots: [] };

const appElement = document.querySelector<HTMLDivElement>("#app");
if (!appElement) throw new Error("Application root is missing");
const app: HTMLDivElement = appElement;

const isDemo = import.meta.env.DEV && new URLSearchParams(location.search).get("demo") === "1";

function icon(name: "radar" | "shield" | "browser" | "terminal" | "check" | "warning"): string {
  const paths = {
    radar: '<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="2"/><path d="M12 4v2M20 12h-2M12 20v-2M4 12h2M12 12l5-5"/>',
    shield: '<path d="M12 3l7 3v5c0 4.6-2.8 8-7 10-4.2-2-7-5.4-7-10V6l7-3z"/><path d="M9 12l2 2 4-4"/>',
    browser: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18M7 6.5h.01M10 6.5h.01"/>',
    terminal: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 9l3 3-3 3M12 15h5"/>',
    check: '<circle cx="12" cy="12" r="9"/><path d="M8 12l3 3 5-6"/>',
    warning: '<path d="M12 3l10 18H2L12 3z"/><path d="M12 9v5M12 17h.01"/>'
  };
  return `<svg class="icon" viewBox="0 0 24 24" aria-hidden="true">${paths[name]}</svg>`;
}

function render(): void {
  const progress = state.phase === "idle" || state.phase === "discovering" ? 1 : state.phase === "select" || state.phase === "inspecting" || state.phase === "verify" ? 2 : state.phase === "installing" ? 3 : state.phase === "ready" ? 4 : 1;
  app.innerHTML = `
    <div class="shell">
      <header class="topbar">
        <div class="brand"><span class="brand-mark">N</span><div><strong>Nav Studio Connector</strong><span>UMEC Space</span></div></div>
        <div class="local-badge"><span></span> Только локальная сеть</div>
      </header>
      <main>
        <section class="hero">
          <p class="eyebrow">Безопасное подключение</p>
          <h1>Найдём робота и подготовим браузер</h1>
          <p>Connector обнаружит Nav Studio, покажет сертификат до установки и откроет проверенное HTTPS-соединение.</p>
        </section>
        ${progressView(progress)}
        <div class="workspace">
          <section class="panel main-panel">${mainView()}</section>
          <aside class="panel guide-panel">${guideView()}</aside>
        </div>
      </main>
      <footer><span>v0.1.0</span><button class="link-button" data-action="diagnose">Скопировать диагностику</button><span>Секреты не сохраняются</span></footer>
    </div>`;
  bindActions();
}

function progressView(active: number): string {
  const labels = ["Поиск", "Проверка", "Сертификат", "Готово"];
  return `<ol class="progress" aria-label="Этапы подключения">${labels.map((label, index) => {
    const step = index + 1;
    const className = step < active ? "done" : step === active ? "active" : "";
    return `<li class="${className}"><span>${step < active ? icon("check") : step}</span><b>${label}</b></li>`;
  }).join("")}</ol>`;
}

function mainView(): string {
  if (state.phase === "idle" || state.phase === "discovering") return discoveryView();
  if (state.phase === "select") return selectionView();
  if (state.phase === "inspecting") return busyView("Проверяем TLS-сертификат…", "Соединяемся напрямую с выбранным роботом. Ошибки сертификата не игнорируются.");
  if (state.phase === "verify" && state.inspection) return verificationView(state.inspection);
  if (state.phase === "installing") return busyView("Устанавливаем сертификат…", "Windows использует хранилище текущего пользователя. Ubuntu покажет один системный запрос прав.");
  if (state.phase === "ready") return readyView();
  return errorView();
}

function discoveryView(): string {
  const busy = state.phase === "discovering";
  return `
    <div class="panel-heading">${icon("radar")}<div><h2>${busy ? "Ищем Nav Studio" : "Найти робота"}</h2><p>${busy ? "Слушаем защищённый DNS-SD профиль до 6 секунд." : "Робот и компьютер должны быть в одной локальной сети."}</p></div></div>
    ${busy ? '<div class="scanner" aria-label="Поиск"><span></span><i></i></div>' : '<button class="primary large" data-action="discover">Начать поиск</button>'}
    <div class="divider"><span>или, если multicast заблокирован</span></div>
    <form class="manual-form" data-form="manual">
      <label for="manual-url">HTTPS-адрес Nav Studio</label>
      <div class="input-action"><input id="manual-url" name="url" type="url" inputmode="url" placeholder="https://agibot-pc2.local:8780/" required /><button class="secondary" type="submit">Проверить</button></div>
      <small>HTTP, адреса с логином/паролем и скрытые перенаправления не принимаются.</small>
    </form>`;
}

function selectionView(): string {
  if (state.robots.length === 0) {
    return `<div class="empty-state">${icon("warning")}<h2>Роботы не найдены</h2><p>Проверьте Wi‑Fi или используйте точный HTTPS-адрес. Сканирования подсети не выполняется.</p><button class="primary" data-action="retry">Повторить поиск</button></div>`;
  }
  return `<div class="panel-heading">${icon("radar")}<div><h2>Выберите робота</h2><p>mDNS показывает кандидатов. Идентичность проверим на следующем шаге.</p></div></div>
    <div class="robot-list">${state.robots.map((robot, index) => `<button class="robot-card" data-robot="${index}"><span class="robot-avatar">R</span><span><b>${escapeHtml(robot.display_name)}</b><small>${escapeHtml(robot.hostname)} · ${escapeHtml(robot.model ?? "модель не указана")}</small><em>${robot.addresses.map(escapeHtml).join(", ")}</em></span><i>Проверить →</i></button>`).join("")}</div>
    <button class="link-button" data-action="retry">Искать ещё раз</button>`;
}

function verificationView(inspection: Inspection): string {
  const suffix = inspection.fingerprint_sha256.slice(-8);
  return `
    <div class="panel-heading">${icon("shield")}<div><h2>Сверьте сертификат робота</h2><p>Сеть обнаружила устройство, но ещё не доказала, что это ваш робот.</p></div></div>
    <div class="identity-grid"><div><span>Адрес</span><b>${escapeHtml(inspection.hostname)}</b></div><div><span>Соединение</span><b>${escapeHtml(inspection.resolved_address)}</b></div><div><span>Субъект</span><b>${escapeHtml(inspection.subject)}</b></div><div><span>Задержка</span><b>${inspection.latency_ms} мс</b></div></div>
    <div class="fingerprint"><span>SHA-256 fingerprint</span><code>${groupFingerprint(inspection.fingerprint_sha256)}</code><button class="copy-button" data-copy="${inspection.fingerprint_sha256}">Копировать</button></div>
    <div class="safety-note">${icon("warning")}<p>Сравните отпечаток с QR/наклейкой робота или данными доверенного администратора. Не подтверждайте значение только потому, что оно показано здесь.</p></div>
    <form data-form="confirm" class="confirm-form"><label for="suffix">Введите последние 8 знаков: <b>${suffix}</b></label><input id="suffix" name="suffix" maxlength="8" autocomplete="off" spellcheck="false" placeholder="${suffix}" required /><label class="checkbox"><input type="checkbox" name="confirmed" required /><span>Я независимо сверил полный отпечаток</span></label><button class="primary large" type="submit">Установить сертификат</button></form>`;
}

function readyView(): string {
  return `<div class="success-state">${icon("check")}<p class="eyebrow">Проверка завершена</p><h2>Nav Studio готова к работе</h2><p>Сертификат установлен, HTTPS-соединение отвечает. Connector не передаёт логины или права управления роботом.</p><button class="primary large" data-action="open">${icon("browser")} Открыть Nav Studio</button><button class="secondary" data-action="instructions">Скопировать инструкцию для агента</button></div>`;
}

function errorView(): string {
  return `<div class="empty-state error">${icon("warning")}<p class="error-code">${escapeHtml(state.error?.code ?? "UNKNOWN_ERROR")}</p><h2>Подключение не завершено</h2><p>${escapeHtml(state.error?.message ?? "Неизвестная ошибка")}</p><div class="button-row"><button class="primary" data-action="retry">Повторить</button><button class="secondary" data-action="diagnose">Скопировать диагностику</button></div></div>`;
}

function busyView(title: string, text: string): string {
  return `<div class="busy-state"><div class="spinner"></div><h2>${title}</h2><p>${text}</p></div>`;
}

function guideView(): string {
  return `<div class="panel-heading compact">${icon("terminal")}<div><h2>Что делает Connector</h2></div></div>
    <ol class="guide-list"><li><span>1</span><div><b>Ищет только Nav Studio</b><p>Запрашивает <code>_umec-nav._tcp.local.</code>, не сканируя всю подсеть.</p></div></li><li><span>2</span><div><b>Проверяет до доверия</b><p>Показывает точный TLS fingerprint и останавливается при несовпадении.</p></div></li><li><span>3</span><div><b>Меняет только trust store</b><p>Не получает SSH, токены робота или права на движение.</p></div></li></ol>
    <div class="agent-card"><span>Для Codex и других агентов</span><code>nav-studio-connector agent describe --json</code><button class="copy-button" data-copy="nav-studio-connector agent describe --json">Копировать команду</button></div>
    <div class="privacy"><b>Граница безопасности</b><p>mDNS — недоверенная подсказка. Автоматическое доверие без подписанного receipt запрещено.</p></div>`;
}

function bindActions(): void {
  document.querySelectorAll<HTMLElement>("[data-action]").forEach((element) => element.addEventListener("click", () => void handleAction(element.dataset.action ?? "")));
  document.querySelectorAll<HTMLElement>("[data-robot]").forEach((element) => element.addEventListener("click", () => {
    const index = Number(element.dataset.robot);
    const robot = state.robots[index];
    if (robot) void inspectRobot(robot);
  }));
  document.querySelectorAll<HTMLElement>("[data-copy]").forEach((element) => element.addEventListener("click", () => void copyText(element.dataset.copy ?? "", element)));
  document.querySelector<HTMLFormElement>("[data-form='manual']")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget as HTMLFormElement);
    const url = String(form.get("url") ?? "").trim();
    void inspectRobot({ robot_id: "manual", display_name: "Ручной адрес", hostname: url, addresses: [], port: 443, studio_url: url });
  });
  document.querySelector<HTMLFormElement>("[data-form='confirm']")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget as HTMLFormElement);
    const expected = state.inspection?.fingerprint_sha256;
    if (!expected || String(form.get("suffix") ?? "").toUpperCase() !== expected.slice(-8)) {
      showInlineError("Последние 8 знаков не совпадают с отпечатком.");
      return;
    }
    void installTrust(expected);
  });
}

async function handleAction(action: string): Promise<void> {
  if (action === "discover" || action === "retry") await discover();
  if (action === "open" && state.inspection) await command("open_studio", { url: state.inspection.studio_url });
  if (action === "instructions") await copyText(agentInstructions(), undefined);
  if (action === "diagnose") await runDiagnostics();
}

async function discover(): Promise<void> {
  state.phase = "discovering";
  state.error = undefined;
  render();
  try {
    state.robots = await command<Robot[]>("discover", { timeoutSeconds: 6 });
    state.phase = "select";
  } catch (error) {
    fail(error);
  }
  render();
}

async function inspectRobot(robot: Robot): Promise<void> {
  state.selected = robot;
  state.phase = "inspecting";
  render();
  try {
    state.inspection = await command<Inspection>("inspect", { url: robot.studio_url });
    state.phase = "verify";
  } catch (error) {
    fail(error);
  }
  render();
}

async function installTrust(expectedFingerprint: string): Promise<void> {
  if (!state.inspection) return;
  state.phase = "installing";
  render();
  try {
    await command("install_trust", { url: state.inspection.studio_url, expectedFingerprint, humanConfirmed: true });
    state.phase = "ready";
  } catch (error) {
    fail(error);
  }
  render();
}

async function runDiagnostics(): Promise<void> {
  try {
    state.diagnostics = await command("diagnostics", { url: state.inspection?.studio_url ?? null, timeoutSeconds: 4 });
    await copyText(JSON.stringify(state.diagnostics, null, 2), undefined);
  } catch (error) {
    fail(error);
    render();
  }
}

async function command<T>(name: string, args: Record<string, unknown>): Promise<T> {
  if (isDemo) return demoCommand<T>(name);
  return invoke<T>(name, args);
}

async function demoCommand<T>(name: string): Promise<T> {
  await new Promise((resolve) => setTimeout(resolve, 450));
  if (name === "discover") return [{ robot_id: "demo-ab12cd", display_name: "Робот AB12CD", hostname: "agibot-pc2.local", addresses: ["192.168.1.42"], port: 8780, studio_url: "https://agibot-pc2.local:8780/", model: "X2 Ultra", api_version: "v1" }] as T;
  if (name === "inspect") return { studio_url: "https://agibot-pc2.local:8780/", hostname: "agibot-pc2.local", resolved_address: "192.168.1.42:8780", fingerprint_sha256: "A1B2C3D4A1B2C3D4A1B2C3D4A1B2C3D4A1B2C3D4A1B2C3D4A1B2C3D4A1B2C3D4", subject: "CN=agibot-pc2.local", issuer: "CN=agibot-pc2.local", not_before: "2026-08-01", not_after: "2027-08-01", latency_ms: 18 } as T;
  return { ok: true } as T;
}

function fail(error: unknown): void {
  const value = (typeof error === "object" && error !== null ? error : { message: String(error) }) as ConnectorError;
  state.error = { code: value.code ?? "CONNECTOR_ERROR", message: value.message ?? String(error), retryable: value.retryable };
  state.phase = "error";
}

async function copyText(value: string, button?: HTMLElement): Promise<void> {
  await navigator.clipboard.writeText(value);
  if (button) {
    const previous = button.textContent;
    button.textContent = "Скопировано";
    setTimeout(() => { button.textContent = previous; }, 1200);
  }
}

function agentInstructions(): string {
  return `Nav Studio Connector CLI contract v1\n1. nav-studio-connector agent describe --json\n2. nav-studio-connector discover --timeout 5 --json\n3. nav-studio-connector inspect --url <https-url> --json\n4. Never trust an unknown fingerprint automatically. Use trust install only with an independently verified SHA-256 value.`;
}

function showInlineError(message: string): void {
  document.querySelector(".confirm-form .inline-error")?.remove();
  const target = document.querySelector(".confirm-form");
  target?.insertAdjacentHTML("afterbegin", `<p class="inline-error">${escapeHtml(message)}</p>`);
}

function groupFingerprint(value: string): string {
  return value.match(/.{1,4}/g)?.join(" ") ?? value;
}

function escapeHtml(value: string): string {
  return value.replace(/[&<>'"]/g, (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;" }[character] ?? character));
}

render();
