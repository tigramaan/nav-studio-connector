# Tool catalog

| Tool | Purpose | Input/output | Failure behavior | REQ |
|---|---|---|---|---|
| `build-windows.ps1` | Reproducible Windows checks and bundles | repo → NSIS/artifacts | stops on first failed check | REQ-012, 013 |
| `build-ubuntu.sh` | Reproducible Ubuntu checks and bundles | repo → DEB/AppImage | strict shell, stops on error | REQ-012, 013 |
| `validate-traceability.mjs` | Verify every REQ appears in traceability | specs → JSON/text result | non-zero on gaps | all |
| `validate-license.mjs` | Verify the public MIT grant and attribution | `LICENSE` → JSON result | non-zero on missing canonical terms/attribution | REQ-018 |
| `scan-secrets.mjs` | Reject common private-key/credential patterns | tracked source → findings | non-zero on match | REQ-013, 014 |
| `visual-qa.mjs` | Exact-viewport Chromium/Edge layout, overflow and console audit with screenshots | URL/output directory → JSON report and PNGs | non-zero on overflow, console error or failed request | REQ-011, 012 |
| `sign-identity-receipt.py` + signer README/requirements | Offline receipt-v1 signer using a private Ed25519 key outside the repository | robot identity/certificate → compact DNS-SD receipt or dry-run JSON | fails on unsafe key location, invalid binding or lifetime | REQ-004, 020 |
| `test-sign-identity-receipt.py` | Exercise real Ed25519 signing, canonical binding and unsafe-key-path rejection | isolated temporary key → unittest result | non-zero on contract or guard failure; deletes temporary key | REQ-004, 013, 020 |
| `verify-windows-signature.ps1` | Verify Authenticode chain, code-signing EKU and optional timestamp | signed EXE files → JSON evidence | non-zero on unsigned/untrusted/untimestamped input | REQ-019 |
| `test-windows-signing-hil.ps1` | Disposable self-signed Authenticode pipeline HIL without modifying release artifacts | final EXE/NSIS → temporary signed copies | always removes test certificates and copies | REQ-019 |
| `test-windows-package-hil.ps1` | Silent current-user NSIS install, installed CLI/GUI launch and uninstall smoke | NSIS package → JSON evidence | refuses a pre-existing installation and always invokes the registered uninstaller | REQ-012, 013 |
| `test-ubuntu-package-hil.sh` | Isolated `.deb`/AppImage, GUI and disposable system-CA lifecycle HIL | packages → JSON evidence | root plus container/ephemeral-host guard; cleanup trap | REQ-008, 009, 012, 015 |
