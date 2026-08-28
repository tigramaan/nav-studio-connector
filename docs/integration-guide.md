# Nav Studio integration guide

Robot-side Nav Studio must advertise `_umec-nav._tcp.local.` with resolved hostname, address and HTTPS port. Allowlisted TXT keys are `schema`, `device_id`, `model`, `api_version`, optional guarded `path`, and the identity fields `identity=receipt-v1` plus `receipt=<compact-value>`. The certificate SAN must contain the advertised hostname. Current legacy deployments may use a self-signed leaf certificate after explicit fingerprint confirmation.

Recommended production evolution:

1. assign a fleet-unique `umec-nav-<short-device-id>.local` hostname;
2. serve HTTPS on 443 with a local Device CA/leaf chain;
3. return authenticated device identity and compare `device_id` after TLS;
4. publish a receipt signed by key `umec-identity-2026`, following `specs/001-desktop-connector/contracts/identity-receipt-v1.md` and binding device ID, normalized hostname, certificate fingerprint and expiry;
5. rotate certificates through a separately authorized receipt rather than silent replacement.

Do not put session tokens, passwords, SSH keys or full serial payloads in DNS-SD TXT records or QR codes.

The public desktop-connector landing page is `https://tigramaan.github.io/nav-studio-connector/`. Link users to GitHub Releases for packages and to `https://tigramaan.github.io/nav-studio-connector/privacy/` for the privacy disclosure; do not mirror unsigned binaries on the static site.

Generate receipts only in a controlled robot provisioning pipeline with `tools/sign-identity-receipt.py`. The Ed25519 private key path must be outside the checkout; production custody should use a restricted vault/HSM-backed signing service. Rotation publishes a new public-key entry before issuance begins, then revokes the previous key only after its bounded receipts expire.
