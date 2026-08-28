#!/usr/bin/env python3
"""Contract tests for the offline identity receipt signer."""

from __future__ import annotations

import base64
import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

ROOT = Path(__file__).resolve().parent.parent
SIGNER = ROOT / "tools" / "sign-identity-receipt.py"
FINGERPRINT = "A" * 64


class SignerContractTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="umec-receipt-test-")
        self.key_path = Path(self.temporary.name) / "identity.pem"
        self.private_key = Ed25519PrivateKey.generate()
        self.key_path.write_bytes(
            self.private_key.private_bytes(
                serialization.Encoding.PEM,
                serialization.PrivateFormat.PKCS8,
                serialization.NoEncryption(),
            )
        )
        if os.name != "nt":
            self.key_path.chmod(0o600)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def arguments(self, key_path: Path) -> list[str]:
        return [
            sys.executable,
            str(SIGNER),
            "--private-key",
            str(key_path),
            "--key-id",
            "test-key",
            "--device-id",
            "robot-test01",
            "--hostname",
            "AGIBOT-TEST.LOCAL.",
            "--fingerprint",
            FINGERPRINT,
            "--expires-unix",
            str(int(time.time()) + 3600),
            "--json",
        ]

    def test_emits_verifiable_bounded_receipt(self) -> None:
        result = subprocess.run(
            self.arguments(self.key_path), capture_output=True, text=True, check=True
        )
        value = json.loads(result.stdout)
        receipt = value["receipt"]
        self.assertLessEqual(len(receipt), 240)
        _, _, expires, fingerprint, signature = receipt.split(".")
        message = (
            f"umec-nav-identity-receipt-v1\0robot-test01\0agibot-test.local\0"
            f"{fingerprint}\0{expires}"
        ).encode()
        padded = signature + "=" * (-len(signature) % 4)
        self.private_key.public_key().verify(base64.urlsafe_b64decode(padded), message)

    def test_rejects_private_key_inside_checkout(self) -> None:
        result = subprocess.run(
            self.arguments(ROOT / "LICENSE") + ["--dry-run"],
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("outside the repository", result.stderr)


if __name__ == "__main__":
    unittest.main()
