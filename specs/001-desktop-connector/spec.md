# Feature Specification: Desktop Connector

**Feature Branch**: `001-desktop-connector`
**Created**: 2026-08-28
**Status**: Approved for implementation
**Input**: Package Nav Studio discovery, certificate trust, human guidance and agent guidance into one Windows and Ubuntu application.

## User scenarios

### US-001 — First connection by a person (P1)

A person launches the connector, sees Nav Studio instances discovered on the local network, selects a robot, verifies its identity, installs the required certificate and opens Nav Studio in the default browser.

**Independent acceptance test**: on a clean workstation, complete the workflow without a terminal and reach a browser page with a valid HTTPS trust chain.

### US-002 — Connection by an automation agent (P1)

An automation agent obtains the supported command contract as JSON, discovers robots, inspects trust state and performs only non-interactive operations whose identity checks can be completed without human judgment.

**Independent acceptance test**: invoke every public CLI command with `--json`, validate the output schema and confirm that an unknown fingerprint is never accepted automatically.

### US-003 — Diagnostics and recovery (P2)

A user can diagnose mDNS, reachability, TLS identity and certificate-store state, copy a sanitized report, remove a certificate installed by the connector and retry discovery using a manual HTTPS address.

**Independent acceptance test**: block mDNS while retaining TCP reachability and complete the manual-address workflow; export a report containing no certificate private key or access secret.

## Requirements

- **REQ-001**: The connector MUST discover `_umec-nav._tcp.local.` services and deduplicate results by stable robot identity.
- **REQ-002**: The connector MUST support a guarded manual `https://` address when multicast discovery is unavailable.
- **REQ-003**: The connector MUST display the robot name, host, addresses, port, Studio URL, identity method and SHA-256 certificate fingerprint before changing trust.
- **REQ-004**: Automatic trust MUST require a valid UMEC-signed identity receipt binding robot identity, host name and certificate fingerprint.
- **REQ-005**: When no verifiable receipt is available, trust installation MUST require explicit human comparison and confirmation of the SHA-256 fingerprint.
- **REQ-006**: Agent mode MUST reject an unknown or mismatched identity rather than prompting, guessing or accepting a fallback.
- **REQ-007**: Windows installation MUST target the current user's trusted root store unless a documented machine-wide mode is explicitly selected.
- **REQ-008**: Ubuntu installation MUST use a narrowly scoped privileged helper flow that writes one PEM/CRT file under `/usr/local/share/ca-certificates/` and executes `update-ca-certificates`.
- **REQ-009**: The connector MUST verify HTTPS health after trust installation before reporting readiness or opening Studio.
- **REQ-010**: The connector MUST expose a stable, versioned JSON CLI contract for agent description, discovery, inspection, trust planning, trust installation, status, open and diagnostics.
- **REQ-011**: The GUI MUST provide human-readable explanations, exact safety consequences, retry paths and copyable agent instructions.
- **REQ-012**: The release workflow MUST produce a Windows installer and Ubuntu `.deb`; AppImage MAY be produced as a portable diagnostic artifact.
- **REQ-013**: Public source and release artifacts MUST contain no SSH private keys, signing private keys, passwords, tokens or customer-specific trust anchors.
- **REQ-014**: Logs and diagnostic exports MUST be structured, bounded and sanitized; certificate public material and fingerprints MAY be logged, private keys MUST NOT.
- **REQ-015**: The connector MUST remove only trust records that it can identify as installed by this connector and MUST show the exact target before removal.
- **REQ-016**: Discovery, HTTPS inspection and health checks MUST have explicit bounded timeouts and deterministic error classes.
- **REQ-017**: The UI MUST remain usable at 1280×720, 1440×900, 1920×1080 and 360×800 without horizontal overflow or hidden primary actions.
- **REQ-018**: The public source distribution MUST include the MIT License with copyright attribution to `tigramaan`.
- **REQ-019**: A tagged Windows release MUST be Authenticode-signed and timestamped with an organization-owned code-signing identity; a release workflow MUST fail closed when signing material is absent.
- **REQ-020**: Signed identity receipt verification MUST use a dedicated pinned Ed25519 trust root with explicit key ID, validity and revocation policy; receipt signing private keys MUST remain outside this repository.

## Security boundary

The local network is untrusted. mDNS records, IP addresses, TXT fields, downloaded certificates and HTTP responses are external input. Discovery proves presence, not identity. A certificate becomes trusted only after verification by a shipped public UMEC root or explicit human fingerprint confirmation. Engineering SSH access is outside this product.

## Observable errors

`DISCOVERY_TIMEOUT`, `INVALID_SERVICE_RECORD`, `INVALID_URL`, `ENDPOINT_UNREACHABLE`, `TLS_FETCH_FAILED`, `IDENTITY_RECEIPT_INVALID`, `IDENTITY_MISMATCH`, `HUMAN_CONFIRMATION_REQUIRED`, `PERMISSION_DENIED`, `TRUST_STORE_FAILED`, `HEALTH_CHECK_FAILED`, `UNSUPPORTED_PLATFORM`.

## Success criteria

- **SC-001**: A first-time Windows user completes the supported flow in at most five visible actions after launch.
- **SC-002**: A first-time Ubuntu user completes the supported flow with at most one system privilege prompt.
- **SC-003**: 100% of trust mutations are preceded by a verified receipt or explicit fingerprint confirmation recorded in the operation result.
- **SC-004**: All public CLI JSON examples validate against the published schema.
- **SC-005**: Windows and Ubuntu smoke tests discover either a real service or a deterministic fixture and classify absence correctly.

## Assumptions and limitations

- Nav Studio advertises `_umec-nav._tcp.local.` and serves HTTPS.
- Existing robots without signed identity receipts use the manual fingerprint confirmation path.
- The project is distributed under the MIT License.
- Code-signing certificates are release-governance inputs and never embedded in source.
- Android is explicitly outside this feature.
