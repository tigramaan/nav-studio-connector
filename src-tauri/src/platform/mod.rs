#[cfg(target_os = "linux")]
mod linux_trust;
#[cfg(target_os = "windows")]
mod windows_trust;

use crate::domain::{ConnectorError, EndpointInspection, ErrorCode, Result};

pub fn platform_name() -> &'static str {
    std::env::consts::OS
}

pub fn install_certificate(inspection: &EndpointInspection) -> Result<String> {
    #[cfg(target_os = "windows")]
    return windows_trust::install_certificate(inspection);
    #[cfg(target_os = "linux")]
    return linux_trust::install_certificate(inspection);
    #[allow(unreachable_code)]
    Err(ConnectorError::new(
        ErrorCode::UnsupportedPlatform,
        "Certificate installation is supported only on Windows and Linux",
        false,
    ))
}

pub fn remove_certificate(
    fingerprint_sha256: &str,
    fingerprint_sha1: &str,
    trust_target: &str,
) -> Result<()> {
    #[cfg(target_os = "windows")]
    return windows_trust::remove_certificate(fingerprint_sha256, fingerprint_sha1, trust_target);
    #[cfg(target_os = "linux")]
    return linux_trust::remove_certificate(fingerprint_sha256, fingerprint_sha1, trust_target);
    #[allow(unreachable_code)]
    Err(ConnectorError::new(
        ErrorCode::UnsupportedPlatform,
        "Certificate removal is supported only on Windows and Linux",
        false,
    ))
}
