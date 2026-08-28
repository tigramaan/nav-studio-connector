use super::*;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::pkcs8::EncodePublicKey;
use ed25519_dalek::{Signer, SigningKey};
use pkcs8::LineEnding;

fn fixture(now: DateTime<Utc>, revoked: bool) -> (String, String) {
    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let public_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let expires = now + Duration::hours(2);
    let fingerprint = "A".repeat(64);
    let message = canonical_message(
        "robot-ab12cd",
        "agibot-pc2.local",
        &fingerprint,
        expires.timestamp(),
    );
    let signature = signing_key.sign(message.as_bytes());
    let receipt = format!(
        "r1.test-key.{}.{}.{}",
        expires.timestamp(),
        fingerprint,
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    );
    let policy = serde_json::json!({
        "schema_version": "1.0",
        "keys": {
            "test-key": {
                "algorithm": "ed25519",
                "usage": "identity_receipt_v1",
                "public_key_pem": public_pem,
                "not_before": now - Duration::hours(1),
                "not_after": now + Duration::days(2),
                "revoked": revoked
            }
        }
    })
    .to_string();
    (receipt, policy)
}

#[test]
fn verifies_bound_signed_receipt() {
    let now = Utc::now();
    let (receipt, policy) = fixture(now, false);
    let verified = verify_with_policy(
        &receipt,
        "robot-ab12cd",
        "AGIBOT-PC2.LOCAL.",
        &"A".repeat(64),
        now,
        &policy,
    )
    .unwrap();
    assert_eq!(verified.key_id, "test-key");
    assert_eq!(verified.hostname, "agibot-pc2.local");
}

#[test]
fn rejects_mismatch_expiry_and_revocation() {
    let now = Utc::now();
    let (receipt, policy) = fixture(now, false);
    for (device_id, hostname, fingerprint, at) in [
        ("robot-other", "agibot-pc2.local", "A".repeat(64), now),
        ("robot-ab12cd", "other.local", "A".repeat(64), now),
        ("robot-ab12cd", "agibot-pc2.local", "B".repeat(64), now),
        (
            "robot-ab12cd",
            "agibot-pc2.local",
            "A".repeat(64),
            now + Duration::days(2),
        ),
    ] {
        assert!(
            verify_with_policy(&receipt, device_id, hostname, &fingerprint, at, &policy).is_err()
        );
    }
    let (_, revoked_policy) = fixture(now, true);
    assert!(verify_with_policy(
        &receipt,
        "robot-ab12cd",
        "agibot-pc2.local",
        &"A".repeat(64),
        now,
        &revoked_policy
    )
    .is_err());
}

#[test]
fn pinned_product_root_verifies_public_test_vector() {
    let now = DateTime::parse_from_rfc3339("2026-08-28T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let receipt = "r1.umec-identity-2026.1788004800.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA.NdDWO6_9D2I5cDNS6VcHqzGCnCEddnNARrEjMTu06aXwqzEVF1Pso0iIpXONAMlrQ6_dwvn6qaJwbthO-bN-Dg";
    let verified = verify_identity_receipt(
        receipt,
        "robot-test01",
        "agibot-test.local",
        &"A".repeat(64),
        now,
    )
    .unwrap();
    assert_eq!(verified.key_id, "umec-identity-2026");
}
