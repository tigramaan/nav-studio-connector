# Identity receipt signer

`sign-identity-receipt.py` is an offline provisioning tool for the public `receipt-v1` contract. It never generates a key inside the checkout and rejects any private-key path located under the repository root.

Install the isolated dependency:

```bash
python3 -m venv /secure/path/receipt-signer-venv
/secure/path/receipt-signer-venv/bin/pip install -r tools/requirements-receipt.txt
```

Generate the Ed25519 key in a restricted external location (or replace this step with an HSM/vault signer):

```bash
umask 077
openssl genpkey -algorithm Ed25519 -out /secure/path/umec_identity_ed25519.pem
```

Inspect the canonical binding without using the key:

```bash
python3 tools/sign-identity-receipt.py --private-key /secure/path/umec_identity_ed25519.pem --key-id umec-identity-2026 --device-id robot-ab12cd --hostname umec-nav-ab12cd.local --fingerprint <64-hex-sha256> --expires-unix <future-unix-time> --dry-run --json
```

Remove `--dry-run` to emit the signed compact receipt. The expiry must be in the future and no more than 366 days away. Publish only the receipt and matching public key; never copy, print, log or commit the private key. Production issuance should audit device ID, hostname, certificate fingerprint, key ID and expiry without recording private material.
