use crate::domain::{
    normalize_fingerprint, ConnectorError, ConnectorStatus, ErrorCode, Result, TrustOperation,
};
use crate::network::DISCOVERY_SERVICE;
use crate::platform;
use std::fs;
use std::path::PathBuf;

fn state_directory() -> Result<PathBuf> {
    let base = dirs::data_local_dir().ok_or_else(|| {
        ConnectorError::new(
            ErrorCode::InternalError,
            "Cannot determine the local application data directory",
            false,
        )
    })?;
    Ok(base.join("UMEC").join("NavStudioConnector"))
}

fn receipt_path(fingerprint_sha256: &str) -> Result<PathBuf> {
    Ok(state_directory()?.join("receipts").join(format!(
        "{}.json",
        normalize_fingerprint(fingerprint_sha256)?
    )))
}

pub fn save_operation_receipt(operation: &TrustOperation) -> Result<()> {
    let path = receipt_path(&operation.fingerprint_sha256)?;
    let parent = path.parent().ok_or_else(|| {
        ConnectorError::new(ErrorCode::InternalError, "Invalid receipt path", false)
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        ConnectorError::new(
            ErrorCode::TrustStoreFailed,
            "Cannot create the operation receipt directory",
            false,
        )
        .with_detail("cause", error.to_string())
    })?;
    let json = serde_json::to_vec_pretty(operation).map_err(|error| {
        ConnectorError::new(
            ErrorCode::InternalError,
            "Cannot serialize the operation receipt",
            false,
        )
        .with_detail("cause", error.to_string())
    })?;
    fs::write(&path, json).map_err(|error| {
        ConnectorError::new(
            ErrorCode::TrustStoreFailed,
            "Cannot store the operation receipt",
            false,
        )
        .with_detail("cause", error.to_string())
    })
}

pub fn remove_owned_certificate(fingerprint_sha256: &str) -> Result<()> {
    let fingerprint = normalize_fingerprint(fingerprint_sha256)?;
    let path = receipt_path(&fingerprint)?;
    let bytes = fs::read(&path).map_err(|_| {
        ConnectorError::new(
            ErrorCode::InvalidInput,
            "No connector-owned installation receipt exists for this fingerprint",
            false,
        )
    })?;
    let receipt: TrustOperation = serde_json::from_slice(&bytes).map_err(|_| {
        ConnectorError::new(
            ErrorCode::TrustStoreFailed,
            "The connector-owned installation receipt is corrupted",
            false,
        )
    })?;
    if receipt.fingerprint_sha256 != fingerprint || !receipt.installed {
        return Err(ConnectorError::new(
            ErrorCode::InvalidInput,
            "Installation receipt does not authorize this removal",
            false,
        ));
    }
    platform::remove_certificate(
        &receipt.fingerprint_sha256,
        &receipt.fingerprint_sha1,
        &receipt.trust_target,
    )?;
    fs::remove_file(path).map_err(|error| {
        ConnectorError::new(
            ErrorCode::TrustStoreFailed,
            "Certificate was removed but its local receipt could not be deleted",
            false,
        )
        .with_detail("cause", error.to_string())
    })
}

pub fn connector_status() -> Result<ConnectorStatus> {
    Ok(ConnectorStatus {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: platform::platform_name().to_string(),
        discovery_service: DISCOVERY_SERVICE.to_string(),
        state_directory: state_directory()?.display().to_string(),
        receipt_verification_configured: false,
    })
}
