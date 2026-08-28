# Research decisions

## Desktop runtime

**Decision**: Tauri 2 with a Rust core and a small TypeScript frontend.
**Rationale**: one domain implementation can serve GUI and CLI while Tauri produces native Windows and Linux packages with a small attack surface.
**Alternatives considered**: Electron (larger runtime), Python/PyInstaller (weaker native packaging and frontend consistency), browser-only page (cannot install OS trust or perform unrestricted mDNS/TLS inspection).

## Identity bootstrap

**Decision**: signed identity receipt for unattended trust; explicit SHA-256 comparison for legacy devices.
**Rationale**: mDNS and a downloaded self-signed certificate do not authenticate a robot.
**Alternatives considered**: engineering SSH retrieval (not distributable), trust-on-first-use without comparison (unsafe on hostile LAN), public Web PKI only (not generally available for `.local`).

## Windows trust scope

**Decision**: CurrentUser root store by default.
**Rationale**: avoids an administrator prompt and limits blast radius.
**Alternatives considered**: LocalMachine root (requires elevation and affects all users), browser-specific stores (inconsistent and difficult to support).

## Ubuntu trust scope

**Decision**: a single named CRT under `/usr/local/share/ca-certificates/` followed by `update-ca-certificates`, invoked through a narrowly scoped privilege prompt.
**Rationale**: integrates with the system trust bundle and Chromium-family browsers.
**Alternatives considered**: application-only trust (browser still warns), running the full GUI as root (unacceptable boundary).

## Distribution

**Decision**: NSIS installer for Windows; `.deb` as the supported Ubuntu artifact; AppImage as an optional portable diagnostic build.
**Rationale**: native install/uninstall semantics and desktop integration, with a portable fallback.
**Alternatives considered**: MSI-only (additional toolchain), archive-only delivery (poor first-run experience).
