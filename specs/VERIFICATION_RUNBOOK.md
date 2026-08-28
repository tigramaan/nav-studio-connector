# Verification runbook

## Prerequisites

- Windows 10/11 x64: Node.js 22+, Rust stable, WebView2 and NSIS dependencies installed by Tauri CLI.
- Ubuntu 22.04/24.04 x64: Node.js 22+, Rust stable and `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev`.
- Trust-store tests use only a disposable generated CA and must finish with certificate and connector receipt removal.

## Reproducible commands

Windows:

```powershell
npm ci --no-audit --no-fund
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
npm run validate:traceability
node tools/scan-secrets.mjs
npm run tauri -- build --bundles nsis
```

Ubuntu core/CLI without desktop libraries:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features
cargo build --manifest-path src-tauri/Cargo.toml --no-default-features --release
./src-tauri/target/release/nav-studio-connector agent describe --json
./src-tauri/target/release/nav-studio-connector discover --timeout 3 --json
```

Ubuntu desktop packages after installing the prerequisites:

```bash
./tools/build-ubuntu.sh
```

Responsive UI (run `npm run dev -- --port 1420 --strictPort` in another terminal):

```powershell
npm run qa:visual -- "http://127.0.0.1:1420/?demo=1" "artifacts/ui-qa-playwright"
```

## Executed evidence — 2026-08-28

| Check | Result | Evidence |
|---|---|---|
| V-001 discovery/manual fallback | PASS | Windows and Ubuntu bounded mDNS discovery returned a valid empty candidate set on the current network; invalid/unreachable manual HTTPS endpoints returned deterministic errors. |
| V-002 identity rejection | PASS | `security_contract`: unknown unattended identity and mismatched expected SHA-256 are rejected. |
| V-003 Windows disposable CA | PASS | Disposable CA installed into `CurrentUser\\Root`, strict HTTPS succeeded, connector-owned removal succeeded; remaining test certificates and receipts: 0. |
| V-003 Ubuntu disposable CA | BLOCKED | VM user has no passwordless package/trust-store elevation. The narrow `pkexec` helper plan is implemented, but system trust mutation was not performed. |
| V-004 post-install HTTPS | PASS (Windows) | Fresh `curl.exe`/Schannel process passed strict TLS after installation; TLS verification was never disabled. |
| V-005 CLI contract | PASS | 17 Windows Rust tests passed, including JSON Schema envelope and exit-code tests; 16 Ubuntu `--no-default-features` tests passed. |
| V-006 responsive UI | PASS | Playwright/Edge exact-viewport and full-page screenshots passed at 320×568, 390×844, 768×1024, 1024×768, 1440×900, 1920×1080, 2560×1440, 3840×2160 and 1440×1600; overflow, undersized controls, console errors and failed requests: 0. |
| V-007 Windows package | PASS | Final release executable and NSIS installer built. Silent current-user install returned 0; installed CLI/GUI launched; standard uninstaller returned 0 and removed its registry entry. |
| V-007 Ubuntu package | PARTIAL | Ubuntu 22.04 VM release CLI built and ran. GitHub Actions built and uploaded `.deb`/AppImage successfully; privileged `.deb` install/launch smoke remains. |
| V-008 secrets/diagnostics | PASS | `scan-secrets.mjs` reported 0 findings; diagnostic contract verifies bounded secret-free output. |
| V-009 timeouts/errors | PASS | Discovery, TLS and HTTPS probes are bounded; unavailable endpoints return `ENDPOINT_UNREACHABLE` with exit code 4. |

## Expected artifacts

- Windows: `src-tauri/target/release/nav-studio-connector.exe` and `src-tauri/target/release/bundle/nsis/*-setup.exe`.
- Ubuntu: `src-tauri/target/release/nav-studio-connector`, `bundle/deb/*.deb` and `bundle/appimage/*.AppImage`.
- Local handoff copies: `artifacts/windows/` and `artifacts/ubuntu/` (gitignored).

## Public CI evidence

- CI run `33159232957`: Windows and Ubuntu jobs completed successfully.
- Package run `33159232917`: Windows and Ubuntu jobs completed successfully.
- Uploaded artifacts: `nav-studio-connector-windows-x64` (1,364,493 bytes) and `nav-studio-connector-ubuntu-x64` (83,076,413 bytes).
- Local final Windows SHA-256: executable `11A649BA260665FB2B8E9885D5D121861CFEF50DE4DADCCCA9F845B0081C71F5`; NSIS installer `DFAC5F337B9E52AC68DBD17EA6C8D085E4159151B2558B72B162808404E7B5B9`.
- Ubuntu VM CLI SHA-256: `FE2373B06971AC0BF026DD5E4DE1A5496D0B544068B36EBC0ED21C31121464C4`.

## Acceptance checklist

- [x] Frontend production build passes.
- [x] Windows Rust tests pass.
- [x] Ubuntu core/CLI tests pass on Ubuntu 22.04 VM.
- [x] Windows NSIS and Ubuntu `.deb`/AppImage packages build in public CI.
- [x] Windows disposable certificate install, strict health and owned removal pass.
- [x] Traceability covers all 17 requirements.
- [x] Source secret scan passes.
- [x] Required responsive viewport screenshots pass.
- [ ] Ubuntu system trust mutation and `.deb` smoke pass in an environment with the documented dependencies/elevation.
- [ ] Windows release is Authenticode-signed with an organization-owned key.

## Known limitations

- The Windows installer is unsigned until an organization-owned code-signing certificate is supplied outside the repository; SmartScreen may warn.
- Automatic signed identity receipts remain fail-closed until the UMEC public verification key is published. Legacy robots require an independently verified SHA-256 fingerprint.
- A repository license has not been selected; source visibility alone does not grant an open-source license.
- The current Ubuntu VM cannot build the desktop bundle or mutate the system trust store without sudo; CI supplies the build dependencies, while trust-store HIL remains a release gate.
