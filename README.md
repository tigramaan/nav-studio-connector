# UMEC Nav Studio Connector

Desktop companion for discovering a local UMEC Nav Studio, verifying its TLS identity, installing an explicitly approved certificate and opening Studio in the browser.

Supported targets:

- Windows 10/11: GUI/CLI executable and per-user NSIS installer;
- Ubuntu 22.04/24.04: GUI/CLI binary, `.deb` package and optional AppImage;
- Android is not part of this repository yet.

## Safety model

mDNS is an untrusted candidate source. The connector never accepts an unknown certificate silently. Trust installation requires either a valid UMEC-signed device identity receipt or an independently verified SHA-256 fingerprint confirmed by a person/caller. The public repository contains no robot SSH credentials or private signing material.

## Use

Launch **UMEC Nav Studio Connector**, select a discovered robot, compare the shown SHA-256 fingerprint with a trusted source, install the certificate and open Studio.

For an automation agent:

```text
nav-studio-connector agent describe --json
nav-studio-connector discover --timeout 5 --json
nav-studio-connector inspect --url https://agibot-pc2.local:8780/ --json
```

An agent may call `trust install` only with an independently trusted expected fingerprint or a valid `receipt-v1` issued by an active pinned UMEC identity key. The active key ID is `umec-identity-2026`; the signing private key is not part of this repository.

## Development

```text
npm ci
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

Build packages with `tools/build-windows.ps1` on Windows or `tools/build-ubuntu.sh` on Ubuntu. See `specs/VERIFICATION_RUNBOOK.md` for platform prerequisites, signing-secret setup and acceptance procedures.

## Project status

Branch builds are intentionally allowed to remain unsigned. Every `v*` tag fails closed unless GitHub receives an organization-owned Authenticode PFX and password as encrypted secrets; both the executable and NSIS installer are then required to have a trusted SHA-256 signature and timestamp. Private keys must never be committed.

## License

Distributed under the [MIT License](LICENSE). Copyright (c) 2026 tigramaan.
