# Implementation Plan: Desktop Connector

## Goal

Deliver a public, reproducible desktop connector that turns local Nav Studio discovery and certificate trust into one guarded workflow for Windows and Ubuntu, with both GUI and machine-readable CLI surfaces.

## Technical context

- **Language**: Rust stable; TypeScript/CSS for the Tauri webview.
- **Desktop runtime**: Tauri 2.
- **Discovery**: DNS-SD/mDNS client for `_umec-nav._tcp.local.`.
- **Network**: bounded TCP/TLS/HTTPS probes; certificate SHA-256 fingerprinting.
- **Persistence**: local non-secret settings and operation receipts only.
- **Packaging**: Tauri NSIS installer on Windows; Debian package and AppImage on Ubuntu.
- **Testing**: Rust unit/integration/contract tests, frontend checks, packaged smoke tests and Ubuntu VM verification.

## Architecture

`frontend -> Tauri commands -> application use cases -> domain policies -> platform/network adapters`

The core library owns identity policy, state transitions and error taxonomy. Platform adapters may inspect or mutate trust stores only through an explicit operation plan. The GUI and CLI call the same application services.

## Constitution check

- Modular responsibilities and dependency direction are explicit.
- Domain identity policy performs no filesystem, process or network I/O.
- External inputs are guarded before use.
- Public CLI and discovery contracts are versioned and contract-tested.
- Every REQ is mapped to a task and verification entry.
- No secrets are stored in source or diagnostic output.

No gate violations are accepted.

## File structure

```text
src/                         frontend UI
src-tauri/src/domain/        pure identity and validation rules
src-tauri/src/application/   discovery/trust/diagnostic use cases
src-tauri/src/platform/      Windows and Ubuntu trust adapters
src-tauri/src/network/       mDNS, TLS and HTTPS adapters
src-tauri/src/bin/           public agent CLI
src-tauri/tests/             integration and contract tests
specs/                       requirements, tasks and verification
docs/                        ADR, architecture, catalogs, operations
tools/                       repeatable build and validation tools
```

## Delivery phases

1. Define requirements, trust policy, state model and public contracts.
2. Implement pure domain guards and deterministic errors.
3. Implement discovery, TLS inspection and health adapters.
4. Implement Windows/Ubuntu trust plans and mutation boundaries.
5. Expose Tauri commands and JSON CLI.
6. Build the responsive guided GUI.
7. Produce platform packages and verify on Windows and Ubuntu VM.
8. Run security, traceability and release checks.

## Test strategy

- Unit: URL/service validation, fingerprint normalization, identity decisions, state transitions.
- Contract: JSON CLI envelope/schema and discovery record format.
- Integration: deterministic local DNS-SD fixture, TLS fixture, trust-plan generation, sanitized diagnostics.
- Trust-store propagation: at most three strict HTTPS health attempts (4 seconds each) with 350/850 ms delays; TLS verification is never disabled.
- Security: unknown identity rejection in agent mode, command argument injection guards, path ownership and secret scan.
- E2E: GUI first-run flow at required viewport sizes; packaged binary smoke tests.
- Platform: Windows CurrentUser trust store and Ubuntu VM `update-ca-certificates` flow using a disposable test CA.

## Acceptance criteria

All REQ rows in `TRACEABILITY_MATRIX.md` have passing automated tests or an executed runbook procedure; release builds contain no secrets; installers launch; an unavailable dependency produces a documented deterministic error.

## Risks

- Multicast may be blocked by Wi-Fi/VLAN policy: manual HTTPS fallback is mandatory.
- Current robots may lack signed receipts: human fingerprint confirmation remains available.
- Branch Windows binaries may show SmartScreen warnings: tagged release fails unless organization Authenticode signing and timestamp verification succeed.
- Linux trust installation requires elevation: only the narrow helper step is privileged.
- Receipt-key loss or misuse affects unattended identity: the private key stays outside source, receipts have bounded lifetime, and the pinned policy supports explicit rotation/revocation.

## Observability

JSON events contain timestamp, operation ID, stage, duration and error code. Network addresses and public fingerprints may appear; tokens, private keys and arbitrary HTTP bodies are excluded. Diagnostic archives are size-bounded.

## Tools/plugins

- Cargo, npm and Tauri CLI for build and packaging.
- Project tools for traceability, schema validation, secret scanning and smoke tests.
- Browser-based viewport verification for GUI layout.

No external account plugin is required.
