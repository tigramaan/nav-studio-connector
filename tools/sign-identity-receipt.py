#!/usr/bin/env python3
"""Create a bounded UMEC identity receipt v1 with an external Ed25519 private key."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import re
import stat
import time
from pathlib import Path

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

CONTEXT = "umec-nav-identity-receipt-v1"
MAX_LIFETIME_SECONDS = 366 * 24 * 60 * 60
SAFE_ID = re.compile(r"^[A-Za-z0-9_-]{6,128}$")
SAFE_KEY_ID = re.compile(r"^[A-Za-z0-9_-]{6,64}$")
FINGERPRINT = re.compile(r"^[A-F0-9]{64}$")


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser(description=__doc__)
    value.add_argument("--private-key", type=Path, required=True)
    value.add_argument("--key-id", required=True)
    value.add_argument("--device-id", required=True)
    value.add_argument("--hostname", required=True)
    value.add_argument("--fingerprint", required=True)
    value.add_argument("--expires-unix", type=int, required=True)
    value.add_argument("--json", action="store_true")
    value.add_argument("--dry-run", action="store_true")
    return value


def main() -> int:
    args = parser().parse_args()
    repository = Path(__file__).resolve().parent.parent
    key_path = args.private_key.expanduser().resolve(strict=True)
    if key_path == repository or repository in key_path.parents:
        raise SystemExit("private signing key must be stored outside the repository")
    if not key_path.is_file():
        raise SystemExit("private signing key path must be a regular file")
    if os.name != "nt" and stat.S_IMODE(key_path.stat().st_mode) & 0o077:
        raise SystemExit("private signing key permissions must deny group and other access")
    if not SAFE_KEY_ID.fullmatch(args.key_id):
        raise SystemExit("invalid key ID")
    if not SAFE_ID.fullmatch(args.device_id):
        raise SystemExit("invalid device ID")
    hostname = args.hostname.strip().rstrip(".").lower()
    if not hostname or len(hostname) > 253 or any(char in hostname for char in "/\\@:\n\r"):
        raise SystemExit("invalid hostname")
    fingerprint = re.sub(r"[^A-Fa-f0-9]", "", args.fingerprint).upper()
    if not FINGERPRINT.fullmatch(fingerprint):
        raise SystemExit("invalid SHA-256 fingerprint")
    remaining = args.expires_unix - int(time.time())
    if remaining <= 0 or remaining > MAX_LIFETIME_SECONDS:
        raise SystemExit("expiry must be in the future and no more than 366 days away")

    message = f"{CONTEXT}\0{args.device_id}\0{hostname}\0{fingerprint}\0{args.expires_unix}".encode()
    if args.dry_run:
        result = {
            "status": "dry-run",
            "key_id": args.key_id,
            "device_id": args.device_id,
            "hostname": hostname,
            "fingerprint_sha256": fingerprint,
            "expires_unix": args.expires_unix,
            "canonical_sha256": hashlib.sha256(message).hexdigest(),
        }
    else:
        try:
            private_key = serialization.load_pem_private_key(key_path.read_bytes(), password=None)
        except (OSError, TypeError, ValueError):
            raise SystemExit("cannot load the unencrypted PEM private signing key") from None
        if not isinstance(private_key, Ed25519PrivateKey):
            raise SystemExit("signing key must be Ed25519")
        signature = base64.urlsafe_b64encode(private_key.sign(message)).decode().rstrip("=")
        receipt = f"r1.{args.key_id}.{args.expires_unix}.{fingerprint}.{signature}"
        if len(receipt) > 240:
            raise SystemExit("receipt exceeds the DNS-SD value bound")
        result = {
            "schema_version": "1.0",
            "receipt": receipt,
            "identity": "receipt-v1",
            "key_id": args.key_id,
            "device_id": args.device_id,
            "hostname": hostname,
            "fingerprint_sha256": fingerprint,
            "expires_unix": args.expires_unix,
        }
    print(json.dumps(result, ensure_ascii=False) if args.json else result.get("receipt", json.dumps(result)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
