# UMEC identity receipt v1

DNS-SD publishes `identity=receipt-v1` and a bounded `receipt` value:

```text
r1.<key_id>.<expires_unix>.<certificate_sha256>.<ed25519_signature_base64url>
```

The Ed25519 signature covers these UTF-8 fields separated by NUL bytes:

```text
umec-nav-identity-receipt-v1\0<device_id>\0<normalized_hostname>\0<uppercase_sha256>\0<expires_unix>
```

Verification guards:

- total receipt length is at most 240 ASCII bytes;
- `device_id`, hostname, key ID, SHA-256 and Unix expiry are validated before cryptography;
- receipt expiry is no more than 366 days ahead and cannot exceed trust-root validity;
- key algorithm is Ed25519, usage is `identity_receipt_v1`, and unknown/revoked/out-of-window keys fail closed;
- signature binds the exact discovered device ID, normalized TLS hostname and observed certificate SHA-256;
- signing private keys remain outside the repository and connector distribution.

Pinned public roots and rotation state are defined in `config/identity-trust-roots.json`. The initial key ID is `umec-identity-2026`.
