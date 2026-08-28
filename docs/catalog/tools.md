# Tool catalog

| Tool | Purpose | Input/output | Failure behavior | REQ |
|---|---|---|---|---|
| `build-windows.ps1` | Reproducible Windows checks and bundles | repo → NSIS/artifacts | stops on first failed check | REQ-012, 013 |
| `build-ubuntu.sh` | Reproducible Ubuntu checks and bundles | repo → DEB/AppImage | strict shell, stops on error | REQ-012, 013 |
| `validate-traceability.mjs` | Verify every REQ appears in traceability | specs → JSON/text result | non-zero on gaps | all |
| `scan-secrets.mjs` | Reject common private-key/credential patterns | tracked source → findings | non-zero on match | REQ-013, 014 |
| `visual-qa.mjs` | Exact-viewport Chromium/Edge layout, overflow and console audit with screenshots | URL/output directory → JSON report and PNGs | non-zero on overflow, console error or failed request | REQ-011, 012 |
