# Agent CLI contract v1

Executable: `nav-studio-connector`.

Commands:

- `agent describe --json`
- `discover --timeout <seconds> --json`
- `inspect --url <https-url> --json`
- `trust plan --url <https-url> --json`
- `trust install --url <https-url> --expected-fingerprint <sha256> --json`
- `trust install --url <https-url> --receipt <receipt-v1> --device-id <id> --json`
- `trust remove --fingerprint <sha256> --json`
- `status --json`
- `open --url <https-url> --json`
- `diagnose [--url <https-url>] --json`

All JSON stdout is one envelope matching `agent-cli.schema.json`. Human progress and diagnostics go to stderr. Exit code is `0` only when `ok=true`; invalid input is `2`; identity/authorization rejection is `3`; unavailable dependency/timeout is `4`; platform mutation failure is `5`.

`trust install` never accepts an observed fingerprint as its own expected value. The expected value must be supplied independently by the caller or derived from a verified signed receipt.

`--expected-fingerprint` and `--receipt` are mutually exclusive. Receipt mode requires `--device-id`; the receipt signature is checked against that ID, the normalized URL hostname and the newly observed TLS certificate fingerprint.
