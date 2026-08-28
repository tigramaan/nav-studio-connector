use super::{normalize_fingerprint, ConnectorError, ErrorCode, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::pkcs8::DecodePublicKey;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const RECEIPT_PREFIX: &str = "r1";
const RECEIPT_CONTEXT: &str = "umec-nav-identity-receipt-v1";
const MAX_RECEIPT_LENGTH: usize = 240;
const MAX_RECEIPT_LIFETIME_DAYS: i64 = 366;
const TRUST_ROOT_POLICY: &str = include_str!("../../../config/identity-trust-roots.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifiedIdentityReceipt {
    pub key_id: String,
    pub device_id: String,
    pub hostname: String,
    pub fingerprint_sha256: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct TrustRootPolicy {
    schema_version: String,
    keys: BTreeMap<String, TrustRoot>,
}

#[derive(Debug, Deserialize)]
struct TrustRoot {
    algorithm: String,
    usage: String,
    public_key_pem: String,
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
    revoked: bool,
}

struct ParsedReceipt {
    key_id: String,
    expires_at: DateTime<Utc>,
    fingerprint_sha256: String,
    signature: Signature,
}

pub fn verify_identity_receipt(
    receipt: &str,
    device_id: &str,
    hostname: &str,
    observed_fingerprint: &str,
    now: DateTime<Utc>,
) -> Result<VerifiedIdentityReceipt> {
    verify_with_policy(
        receipt,
        device_id,
        hostname,
        observed_fingerprint,
        now,
        TRUST_ROOT_POLICY,
    )
}

pub fn active_identity_key_ids(now: DateTime<Utc>) -> Vec<String> {
    let Ok(policy) = parse_policy(TRUST_ROOT_POLICY) else {
        return Vec::new();
    };
    policy
        .keys
        .into_iter()
        .filter(|(_, key)| !key.revoked && key.not_before <= now && now <= key.not_after)
        .map(|(key_id, _)| key_id)
        .collect()
}

fn verify_with_policy(
    receipt: &str,
    device_id: &str,
    hostname: &str,
    observed_fingerprint: &str,
    now: DateTime<Utc>,
    policy_json: &str,
) -> Result<VerifiedIdentityReceipt> {
    validate_device_id(device_id)?;
    let hostname = normalize_hostname(hostname)?;
    let observed_fingerprint = normalize_fingerprint(observed_fingerprint)?;
    let parsed = parse_receipt(receipt)?;
    if parsed.fingerprint_sha256 != observed_fingerprint {
        return Err(
            receipt_error("Receipt fingerprint does not match the observed certificate")
                .with_detail("receipt_fingerprint", parsed.fingerprint_sha256)
                .with_detail("observed_fingerprint", observed_fingerprint),
        );
    }
    if parsed.expires_at <= now {
        return Err(receipt_error("Identity receipt has expired"));
    }
    if parsed.expires_at > now + Duration::days(MAX_RECEIPT_LIFETIME_DAYS) {
        return Err(receipt_error(
            "Identity receipt lifetime exceeds the allowed bound",
        ));
    }

    let policy = parse_policy(policy_json)?;
    let root = policy
        .keys
        .get(&parsed.key_id)
        .ok_or_else(|| receipt_error("Identity receipt key ID is not trusted"))?;
    if root.revoked {
        return Err(receipt_error("Identity receipt key has been revoked"));
    }
    if root.algorithm != "ed25519" || root.usage != "identity_receipt_v1" {
        return Err(receipt_error(
            "Identity trust root has an unsupported policy",
        ));
    }
    if now < root.not_before || now > root.not_after || parsed.expires_at > root.not_after {
        return Err(receipt_error(
            "Identity trust root is outside its validity period",
        ));
    }
    let public_key = VerifyingKey::from_public_key_pem(&root.public_key_pem)
        .map_err(|_| receipt_error("Identity trust root public key is invalid"))?;
    let message = canonical_message(
        device_id,
        &hostname,
        &parsed.fingerprint_sha256,
        parsed.expires_at.timestamp(),
    );
    public_key
        .verify_strict(message.as_bytes(), &parsed.signature)
        .map_err(|_| receipt_error("Identity receipt signature is invalid"))?;

    Ok(VerifiedIdentityReceipt {
        key_id: parsed.key_id,
        device_id: device_id.to_string(),
        hostname,
        fingerprint_sha256: parsed.fingerprint_sha256,
        expires_at: parsed.expires_at,
    })
}

fn parse_receipt(receipt: &str) -> Result<ParsedReceipt> {
    if receipt.is_empty() || receipt.len() > MAX_RECEIPT_LENGTH || !receipt.is_ascii() {
        return Err(receipt_error(
            "Identity receipt is empty or exceeds the DNS-SD bound",
        ));
    }
    let parts: Vec<&str> = receipt.split('.').collect();
    if parts.len() != 5 || parts[0] != RECEIPT_PREFIX {
        return Err(receipt_error("Identity receipt format is invalid"));
    }
    validate_key_id(parts[1])?;
    let expires_unix = parts[2]
        .parse::<i64>()
        .map_err(|_| receipt_error("Identity receipt expiry is invalid"))?;
    let expires_at = DateTime::from_timestamp(expires_unix, 0)
        .ok_or_else(|| receipt_error("Identity receipt expiry is outside the supported range"))?;
    let fingerprint_sha256 = normalize_fingerprint(parts[3])?;
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(parts[4])
        .map_err(|_| receipt_error("Identity receipt signature encoding is invalid"))?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| receipt_error("Identity receipt signature length is invalid"))?;
    Ok(ParsedReceipt {
        key_id: parts[1].to_string(),
        expires_at,
        fingerprint_sha256,
        signature,
    })
}

fn parse_policy(value: &str) -> Result<TrustRootPolicy> {
    let policy: TrustRootPolicy = serde_json::from_str(value)
        .map_err(|_| receipt_error("Identity trust-root policy is invalid"))?;
    if policy.schema_version != "1.0" || policy.keys.is_empty() {
        return Err(receipt_error(
            "Identity trust-root policy is unsupported or empty",
        ));
    }
    Ok(policy)
}

fn validate_device_id(value: &str) -> Result<()> {
    if !(6..=128).contains(&value.len())
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(receipt_error("Identity receipt device ID is invalid"));
    }
    Ok(())
}

fn validate_key_id(value: &str) -> Result<()> {
    if !(6..=64).contains(&value.len())
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(receipt_error("Identity receipt key ID is invalid"));
    }
    Ok(())
}

fn normalize_hostname(value: &str) -> Result<String> {
    let hostname = value.trim().trim_end_matches('.').to_ascii_lowercase();
    if hostname.is_empty()
        || hostname.len() > 253
        || !hostname.is_ascii()
        || hostname.contains(['/', '\\', '@', ':'])
    {
        return Err(receipt_error("Identity receipt hostname is invalid"));
    }
    Ok(hostname)
}

fn canonical_message(
    device_id: &str,
    hostname: &str,
    fingerprint_sha256: &str,
    expires_unix: i64,
) -> String {
    format!("{RECEIPT_CONTEXT}\0{device_id}\0{hostname}\0{fingerprint_sha256}\0{expires_unix}")
}

fn receipt_error(message: &'static str) -> ConnectorError {
    ConnectorError::new(ErrorCode::IdentityReceiptInvalid, message, false)
}

#[cfg(test)]
#[path = "identity_receipt_tests.rs"]
mod tests;
