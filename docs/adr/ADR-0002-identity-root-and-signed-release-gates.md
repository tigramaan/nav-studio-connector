# ADR-0002: Dedicated identity root and signed release gates

## Context

Unattended certificate installation cannot treat mDNS as identity. The public connector also needs reproducible packages without publishing either the robot-identity signing key or the Windows Authenticode key.

## Decision

Identity receipt v1 uses Ed25519 and a dedicated pinned public-root policy. A receipt binds context string, device ID, normalized hostname, observed certificate SHA-256 and Unix expiry. The policy selects the key by ID and checks algorithm, usage, validity and revocation before strict signature verification. The initial key ID is `umec-identity-2026`; its private key is generated and stored outside the repository.

Windows branch artifacts may be unsigned development builds. Every `v*` tag must import an organization-owned PFX from encrypted CI secrets into the ephemeral user store, configure SHA-256 Authenticode and timestamping, verify the application executable and NSIS installer, then remove imported material. Missing, malformed, untrusted or untimestamped inputs fail the job.

Ubuntu package acceptance runs on an explicitly ephemeral privileged host. It installs the real `.deb`, starts the GUI under a virtual display, exercises CLI JSON, installs a disposable CA through the same adapter, proves strict HTTPS trust, removes the connector-owned CA and proves trust is gone.

## Alternatives

- Reusing the Nav Studio release-update key was rejected because identity and update signing require separate blast radii and rotation.
- Embedding a private key was rejected because public source and client packages are untrusted distribution surfaces.
- Accepting unsigned tagged Windows packages was rejected because it makes publisher identity and release integrity unverifiable.
- Mocking Ubuntu trust mutation was rejected because package paths, privilege behavior and CA propagation require system-level evidence.

## Trade-offs and consequences

Robot provisioning must securely invoke an external signer and rotate keys through policy updates. A real Windows release remains dependent on a CA-issued organizational certificate and CI secret custody. Branch builds stay convenient, while tagged artifacts and unattended trust fail closed. HIL cleanup is restricted to exact connector-owned certificate names and disposable environments.

## Traceability

- Requirements: REQ-004, REQ-008, REQ-009, REQ-012, REQ-013, REQ-015, REQ-019, REQ-020.
- Tasks: T035, T036, T037.
- Verification: V-003, V-004, V-007, V-011, V-012.
