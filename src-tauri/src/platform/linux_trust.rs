use crate::domain::{ConnectorError, EndpointInspection, ErrorCode, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

const CERTIFICATE_DIRECTORY: &str = "/usr/local/share/ca-certificates";

pub fn install_certificate(inspection: &EndpointInspection) -> Result<String> {
    let suffix = &inspection.fingerprint_sha256[..16];
    let target = format!("{CERTIFICATE_DIRECTORY}/umec-nav-studio-{suffix}.crt");
    let certificate_file = NamedTempFile::new().map_err(|error| {
        ConnectorError::new(
            ErrorCode::TrustStoreFailed,
            "Cannot create a temporary certificate file",
            false,
        )
        .with_detail("cause", error.to_string())
    })?;
    fs::write(
        certificate_file.path(),
        inspection.certificate_pem.as_bytes(),
    )
    .map_err(|error| {
        ConnectorError::new(
            ErrorCode::TrustStoreFailed,
            "Cannot prepare the public certificate for installation",
            false,
        )
        .with_detail("cause", error.to_string())
    })?;
    let script =
        "set -eu; /usr/bin/install -m 0644 -- \"$1\" \"$2\"; /usr/sbin/update-ca-certificates";
    let output = privileged_command(script)
        .arg(certificate_file.path())
        .arg(&target)
        .output()
        .map_err(|error| {
            ConnectorError::new(
                ErrorCode::TrustStoreFailed,
                "Cannot start the Ubuntu privilege prompt",
                false,
            )
            .with_detail("cause", error.to_string())
        })?;
    if !output.status.success() {
        return Err(classify_privileged_failure(&output.stderr));
    }
    Ok(target)
}

pub fn remove_certificate(
    fingerprint_sha256: &str,
    _fingerprint_sha1: &str,
    trust_target: &str,
) -> Result<()> {
    let expected = format!(
        "{CERTIFICATE_DIRECTORY}/umec-nav-studio-{}.crt",
        &fingerprint_sha256[..16]
    );
    if trust_target != expected || !Path::new(trust_target).starts_with(CERTIFICATE_DIRECTORY) {
        return Err(ConnectorError::new(
            ErrorCode::InvalidInput,
            "Certificate removal target is not connector-owned",
            false,
        ));
    }
    let script = "set -eu; /usr/bin/rm -f -- \"$1\"; /usr/sbin/update-ca-certificates";
    let output = privileged_command(script)
        .arg(trust_target)
        .output()
        .map_err(|error| {
            ConnectorError::new(
                ErrorCode::TrustStoreFailed,
                "Cannot start the Ubuntu privilege prompt",
                false,
            )
            .with_detail("cause", error.to_string())
        })?;
    if !output.status.success() {
        return Err(classify_privileged_failure(&output.stderr));
    }
    Ok(())
}

fn privileged_command(script: &str) -> Command {
    // Root is used only by isolated package/HIL environments. Desktop users always cross the
    // explicit pkexec boundary; arguments are appended separately and never interpolated.
    let is_root = unsafe { libc::geteuid() == 0 };
    let mut command = if is_root {
        Command::new("/bin/sh")
    } else {
        let mut command = Command::new("pkexec");
        command.arg("/bin/sh");
        command
    };
    command.args(["-c", script, "nav-studio-connector"]);
    command
}

fn classify_privileged_failure(stderr: &[u8]) -> ConnectorError {
    let message = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    let code = if message.contains("not authorized")
        || message.contains("dismissed")
        || message.contains("permission")
    {
        ErrorCode::PermissionDenied
    } else {
        ErrorCode::TrustStoreFailed
    };
    ConnectorError::new(
        code,
        "Ubuntu rejected the certificate-store operation",
        false,
    )
}
