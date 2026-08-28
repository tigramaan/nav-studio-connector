# Module catalog

| Module | Responsibility | Public interface | Dependencies | State/errors | REQ / tests |
|---|---|---|---|---|---|
| `domain` | Pure validation, Ed25519 receipt verification and trust authorization | Rust types/functions and pinned public-root policy | pure crypto/serialization libraries | deterministic `ConnectorError`; invalid/expired/mismatched/revoked receipts fail closed | REQ-003–006, 016, 020 / unit and public-vector tests |
| `network` | mDNS, TLS inspection and HTTPS health | application-internal functions | OS sockets/TLS | bounded timeouts | REQ-001–003, 009, 016 / integration tests |
| `platform` | Trust-store mutation | application-internal adapter | Windows certificate APIs or fixed `pkexec`/root-HIL command | permission/store errors | REQ-007, 008, 015 / platform HIL runbook |
| `application` | Orchestration and receipts | Tauri/CLI callers | domain/network/platform | explicit operation receipt | REQ-009, 014, 015 / workflow tests |
| `cli` | Public machine contract | JSON CLI v1 | application | defined exit codes | REQ-006, 010 / contract test |
