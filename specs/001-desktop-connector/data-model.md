# Data model

## DiscoveredRobot

- `robot_id: string` — stable validated identifier.
- `display_name: string` — untrusted display value, length-bounded.
- `service_name: string` — DNS-SD instance.
- `hostname: string` — normalized DNS host.
- `addresses: string[]` — validated IPv4/IPv6 addresses.
- `port: u16` — non-zero TCP port.
- `studio_url: https URL` — derived or validated TXT value.
- `txt: map<string,string>` — allowlisted, length-bounded fields only.
- `last_seen_at: RFC3339 timestamp`.

## EndpointInspection

- endpoint identity and resolved address;
- leaf/CA public certificate PEM;
- SHA-256 fingerprint;
- certificate subject, issuer, SAN and validity;
- receipt status: `valid | missing | invalid | mismatched`;
- trust state: `trusted | untrusted | expired | unknown`;
- health state and bounded latency.

## TrustDecision

States: `unverified -> receipt_verified | human_confirmation_required -> authorized -> installed -> health_verified`.

Terminal failure states: `rejected`, `identity_mismatch`, `permission_denied`, `install_failed`, `health_failed`.

Rules:

- `authorized` requires a valid receipt or a human confirmation matching the observed fingerprint.
- Agent mode can transition to `authorized` only from `receipt_verified` or an exact caller-supplied expected fingerprint.
- A mismatch always transitions to `identity_mismatch`; no retry changes the expected value implicitly.

## OperationReceipt

- `operation_id`, `timestamp`, `app_version`, `platform`;
- `robot_id`, `hostname`, `fingerprint`;
- authorization method;
- exact trust target;
- result and error code;
- no secrets or private certificate material.

## ConnectorStatus

- app/platform versions;
- discovery capability;
- selected robot;
- inspection/trust/health states;
- next allowed actions;
- last sanitized error.
