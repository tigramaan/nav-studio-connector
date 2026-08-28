# Tasks: Desktop Connector

## Phase 1 — Setup

- [x] T001 Initialize Tauri 2, TypeScript and Rust workspace files in `package.json`, `src/` and `src-tauri/`
- [x] T002 Add reproducible developer/build commands in `tools/` and CI workflow in `.github/workflows/ci.yml`
- [x] T003 Add architecture, ADR and catalog documents in `docs/`

## Phase 2 — Foundational

- [x] T004 [P] Implement guarded models and error taxonomy in `src-tauri/src/domain/`
- [x] T005 [P] Implement structured sanitized diagnostics in `src-tauri/src/application/diagnostics.rs`
- [x] T006 Implement application service interfaces in `src-tauri/src/application/`
- [x] T007 [P] Add core unit and contract tests in `src-tauri/src/` and `src-tauri/tests/`

## Phase 3 — US-001 First connection by a person

- [x] T008 [P] [US1] Add discovery contract tests in `src-tauri/tests/discovery_contract.rs`
- [x] T009 [P] [US1] Add identity-policy tests in `src-tauri/src/domain/identity.rs` and `src-tauri/tests/security_contract.rs`
- [x] T010 [US1] Implement mDNS discovery and manual URL validation in `src-tauri/src/network/discovery.rs`
- [x] T011 [US1] Implement TLS certificate inspection and bounded health probes in `src-tauri/src/network/inspection.rs`
- [x] T012 [P] [US1] Implement Windows CurrentUser trust adapter in `src-tauri/src/platform/windows_trust.rs`
- [x] T013 [P] [US1] Implement Ubuntu trust adapter and helper plan in `src-tauri/src/platform/linux_trust.rs`
- [x] T014 [US1] Implement guarded trust workflow in `src-tauri/src/application/trust.rs`
- [x] T015 [US1] Expose Tauri commands in `src-tauri/src/lib.rs`
- [x] T016 [US1] Build the responsive guided workflow in `index.html`, `src/main.ts` and `src/styles.css`
- [x] T017 [US1] Verify first-run UI and post-install HTTPS health at required viewports

## Phase 4 — US-002 Connection by an automation agent

- [x] T018 [P] [US2] Add CLI JSON schema tests in `src-tauri/tests/agent_cli_contract.rs`
- [x] T019 [US2] Implement versioned JSON envelopes and exit codes in `src-tauri/src/cli.rs` and `src-tauri/src/main.rs`
- [x] T020 [US2] Implement `agent describe`, discovery, inspection, trust, status, open and diagnostic commands
- [x] T021 [US2] Add agent instruction copy/export UI in `src/main.ts`
- [x] T022 [US2] Verify that unattended unknown identity is rejected

## Phase 5 — US-003 Diagnostics and recovery

- [x] T023 [P] [US3] Add sanitized diagnostic snapshot tests in `src-tauri/tests/diagnostics_contract.rs`
- [x] T024 [US3] Implement bounded diagnostic report and copy/export flow in `src-tauri/src/application/diagnostics.rs` and `src/main.ts`
- [x] T025 [US3] Implement connector-owned certificate removal in platform adapters
- [x] T026 [US3] Verify mDNS-empty manual-address validation and deterministic recovery diagnostics

## Final phase — Packaging and hardening

- [x] T027 Configure Windows NSIS, Ubuntu DEB and AppImage bundles in `src-tauri/tauri.conf.json`
- [x] T028 Add Windows and Ubuntu packaging tools in `tools/build-windows.ps1` and `tools/build-ubuntu.sh`
- [x] T029 Run Windows tests and packaged installer smoke test
- [x] T030 Run Ubuntu VM core/CLI tests and ephemeral privileged disposable-CA/`.deb`/GUI/AppImage smoke test
- [x] T031 Run secret scan, traceability validation and source-package inspection
- [x] T032 Update `specs/VERIFICATION_RUNBOOK.md`, catalogs and handover with executed evidence
- [x] T033 Commit and publish the verified feature branch
- [x] T034 Add the MIT License and public attribution
- [x] T035 Implement and contract-test signed identity receipt v1 with a dedicated pinned Ed25519 trust root
- [x] T036 Add fail-closed tagged Windows Authenticode signing and verification workflow
- [x] T037 Complete isolated Ubuntu `.deb` install/GUI and disposable-CA trust-store HIL
- [x] T038 Implement the static project homepage and privacy policy in `site/`
- [x] T039 Add the least-privilege GitHub Pages deployment workflow
- [ ] T040 Verify the published website across the required production viewport matrix

## Dependencies

Setup → Foundational → US-001. US-002 and US-003 reuse the foundational core and can proceed after the corresponding application APIs exist. Packaging follows all user stories; platform builds are independent of each other.

## Independent acceptance

- **US-001**: GUI discovers/selects a fixture robot, verifies identity, installs a disposable CA and passes HTTPS health.
- **US-002**: every public command emits schema-valid JSON; unknown identity returns exit code 3.
- **US-003**: with multicast unavailable, manual URL diagnostics work and the exported report contains no secret material.

## MVP

Phases 1–3 plus Windows and Ubuntu development builds constitute the first usable MVP. Agent CLI, diagnostics and signed installers complete the distribution-ready feature.
