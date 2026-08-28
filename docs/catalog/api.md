# API catalog

The public API is the JSON CLI v1 in `specs/001-desktop-connector/contracts/agent-cli.md`. Its envelope is schema-validated and backward-compatible within major version 1. Tauri commands are internal UI boundaries and may evolve with the bundled frontend.

DNS-SD integration is public and documented in `specs/001-desktop-connector/contracts/discovery.md`. Breaking changes require a new schema version and ADR.

Signed automatic identity is the public `identity-receipt-v1.md` contract. Verification is fail-closed for invalid, expired, mismatched, unknown or revoked Ed25519 receipts.
