# ADR-0001: Desktop runtime and local trust bootstrap

**Status**: accepted
**Date**: 2026-08-28
**Requirements**: REQ-001–REQ-017

## Context

A browser page cannot reliably enumerate DNS-SD services or modify operating-system trust stores. Windows and Ubuntu users need one launchable application, while agents need a deterministic machine-readable interface.

## Decision

Use Tauri 2 with a Rust core and TypeScript UI. Ship one binary that opens the GUI without arguments and exposes the versioned JSON CLI when arguments are supplied. Discovery is never identity. Automatic trust requires a signed receipt; legacy devices use independently compared SHA-256 fingerprints. Windows defaults to CurrentUser Root. Ubuntu elevates only a fixed certificate-copy/update operation.

## Alternatives

- Browser-only/PWA: rejected because mDNS and OS trust are unavailable or inconsistent.
- Electron: viable but materially larger runtime and update surface.
- Python/PyInstaller: viable prototype path but less consistent native packaging and UI.
- SSH certificate retrieval: rejected for public clients because it would distribute engineering credentials.

## Consequences

Each operating system must build its own artifact. Linux requires WebKitGTK and a privilege broker. Windows release reputation requires external code signing. Receipt verification remains fail-closed until the UMEC public verification key and robot-side receipt contract are provisioned.
