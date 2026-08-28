use super::state::save_operation_receipt;
use crate::domain::{
    authorize_trust, verify_identity_receipt, CallerMode, HealthState, Result, TrustOperation,
};
use crate::network::{check_health, inspect_endpoint};
use crate::platform;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::thread;
use std::time::Duration;

pub fn install_endpoint_trust(
    url: &str,
    expected_fingerprint: Option<&str>,
    identity_receipt: Option<&str>,
    device_id: Option<&str>,
    human_confirmed: bool,
    caller_mode: CallerMode,
) -> Result<TrustOperation> {
    let inspection = inspect_endpoint(url, 8)?;
    let receipt_verified = match identity_receipt {
        Some(receipt) => {
            let device_id = device_id.ok_or_else(|| {
                crate::domain::ConnectorError::new(
                    crate::domain::ErrorCode::IdentityReceiptInvalid,
                    "Identity receipt verification requires a device ID",
                    false,
                )
            })?;
            verify_identity_receipt(
                receipt,
                device_id,
                &inspection.hostname,
                &inspection.fingerprint_sha256,
                Utc::now(),
            )?;
            true
        }
        None => false,
    };
    let authorization = authorize_trust(
        &inspection.fingerprint_sha256,
        expected_fingerprint,
        receipt_verified,
        human_confirmed,
        caller_mode,
    )?;
    let target = platform::install_certificate(&inspection)?;
    let digest = Sha256::digest(format!(
        "{}|{}|{}",
        inspection.hostname,
        inspection.fingerprint_sha256,
        Utc::now().timestamp_millis()
    ));
    let mut operation = TrustOperation {
        operation_id: format!("trust-{}", &hex::encode(digest)[..16]),
        timestamp: Utc::now(),
        platform: platform::platform_name().to_string(),
        hostname: inspection.hostname,
        fingerprint_sha256: inspection.fingerprint_sha256,
        fingerprint_sha1: inspection.fingerprint_sha1,
        authorization_method: authorization.method,
        trust_target: target,
        installed: true,
        health_state: HealthState::Unknown,
    };
    if let Err(error) = save_operation_receipt(&operation) {
        let _ = platform::remove_certificate(
            &operation.fingerprint_sha256,
            &operation.fingerprint_sha1,
            &operation.trust_target,
        );
        return Err(error);
    }
    match check_health_after_trust_update(url) {
        Ok(health_state) => operation.health_state = health_state,
        Err(error) => {
            operation.health_state = HealthState::Unhealthy;
            save_operation_receipt(&operation)?;
            return Err(error
                .with_detail("certificate_installed", true)
                .with_detail("fingerprint_sha256", operation.fingerprint_sha256.clone()));
        }
    }
    save_operation_receipt(&operation)?;
    Ok(operation)
}

fn check_health_after_trust_update(url: &str) -> Result<HealthState> {
    let delays = [Duration::from_millis(350), Duration::from_millis(850)];
    let mut last_error = None;
    for attempt in 1..=3 {
        match check_health(url, 4) {
            Ok(state) => return Ok(state),
            Err(error) => last_error = Some(error),
        }
        if let Some(delay) = delays.get(attempt - 1) {
            thread::sleep(*delay);
        }
    }
    Err(last_error
        .expect("bounded health loop always records an error")
        .with_detail("health_attempts", 3))
}
