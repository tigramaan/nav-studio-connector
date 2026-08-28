use crate::domain::{
    validate_https_url, ConnectorError, EndpointInspection, ErrorCode, HealthState, IdentityState,
    Result, TrustState,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use native_tls::TlsConnector;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
#[cfg(target_os = "windows")]
use std::process::Command;
use std::time::{Duration, Instant};
use x509_parser::parse_x509_certificate;

pub fn inspect_endpoint(value: &str, timeout_seconds: u64) -> Result<EndpointInspection> {
    let parsed = validate_https_url(value)?;
    let host = parsed
        .host_str()
        .ok_or_else(|| ConnectorError::new(ErrorCode::InvalidUrl, "URL host is missing", false))?;
    let port = parsed.port_or_known_default().unwrap_or(443);
    let timeout = Duration::from_secs(timeout_seconds.clamp(1, 30));
    let addresses: Vec<SocketAddr> = (host, port)
        .to_socket_addrs()
        .map_err(|error| {
            ConnectorError::new(
                ErrorCode::EndpointUnreachable,
                "Cannot resolve the Nav Studio host",
                true,
            )
            .with_detail("cause", error.to_string())
        })?
        .collect();
    if addresses.is_empty() {
        return Err(ConnectorError::new(
            ErrorCode::EndpointUnreachable,
            "Nav Studio host resolved to no addresses",
            true,
        ));
    }

    let started = Instant::now();
    let (tcp, address) = connect_any(&addresses, timeout)?;
    tcp.set_read_timeout(Some(timeout)).ok();
    tcp.set_write_timeout(Some(timeout)).ok();
    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .map_err(|error| {
            ConnectorError::new(
                ErrorCode::TlsFetchFailed,
                "Cannot initialize TLS inspection",
                false,
            )
            .with_detail("cause", error.to_string())
        })?;
    let stream = connector.connect(host, tcp).map_err(|error| {
        ConnectorError::new(
            ErrorCode::TlsFetchFailed,
            "TLS handshake failed while reading the public certificate",
            true,
        )
        .with_detail("cause", error.to_string())
    })?;
    let certificate = stream
        .peer_certificate()
        .map_err(|error| {
            ConnectorError::new(
                ErrorCode::TlsFetchFailed,
                "Cannot read the peer certificate",
                false,
            )
            .with_detail("cause", error.to_string())
        })?
        .ok_or_else(|| {
            ConnectorError::new(
                ErrorCode::TlsFetchFailed,
                "Peer did not provide a certificate",
                false,
            )
        })?;
    let der = certificate.to_der().map_err(|error| {
        ConnectorError::new(
            ErrorCode::TlsFetchFailed,
            "Cannot decode the peer certificate",
            false,
        )
        .with_detail("cause", error.to_string())
    })?;
    if der.len() > 1024 * 1024 {
        return Err(ConnectorError::new(
            ErrorCode::TlsFetchFailed,
            "Peer certificate exceeds the 1 MiB safety limit",
            false,
        ));
    }

    let fingerprint_sha256 = hex::encode_upper(Sha256::digest(&der));
    let fingerprint_sha1 = hex::encode_upper(Sha1::digest(&der));
    let (subject, issuer, not_before, not_after) = certificate_metadata(&der);

    Ok(EndpointInspection {
        studio_url: parsed.to_string(),
        hostname: host.to_string(),
        resolved_address: address.to_string(),
        certificate_pem: to_pem(&der),
        fingerprint_sha256,
        fingerprint_sha1,
        subject,
        issuer,
        not_before,
        not_after,
        identity_state: IdentityState::HumanConfirmationRequired,
        trust_state: TrustState::Unknown,
        inspected_at: Utc::now(),
        latency_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
    })
}

pub fn check_health(value: &str, timeout_seconds: u64) -> Result<HealthState> {
    let parsed = validate_https_url(value)?;
    #[cfg(target_os = "windows")]
    return check_health_with_fresh_schannel_process(parsed.as_str(), timeout_seconds);

    #[cfg(not(target_os = "windows"))]
    {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds.clamp(1, 30)))
            .build()
            .map_err(|error| {
                ConnectorError::new(
                    ErrorCode::HealthCheckFailed,
                    "Cannot initialize HTTPS health client",
                    false,
                )
                .with_detail("cause", error.to_string())
            })?;
        let response = client.get(parsed).send().map_err(|error| {
            ConnectorError::new(
                ErrorCode::HealthCheckFailed,
                "Nav Studio HTTPS health request failed",
                true,
            )
            .with_detail("cause", error.to_string())
        })?;
        if response.status().is_success()
            || response.status().is_redirection()
            || response.status().as_u16() == 401
            || response.status().as_u16() == 403
        {
            Ok(HealthState::Healthy)
        } else {
            Err(ConnectorError::new(
                ErrorCode::HealthCheckFailed,
                format!("Nav Studio returned HTTP {}", response.status().as_u16()),
                true,
            ))
        }
    }
}

#[cfg(target_os = "windows")]
fn check_health_with_fresh_schannel_process(
    value: &str,
    timeout_seconds: u64,
) -> Result<HealthState> {
    let timeout = timeout_seconds.clamp(1, 30).to_string();
    let output = Command::new("curl.exe")
        .args([
            "--silent",
            "--show-error",
            "--noproxy",
            "*",
            "--max-time",
            &timeout,
            "--output",
            "NUL",
            "--write-out",
            "%{http_code}",
            value,
        ])
        .output()
        .map_err(|error| {
            ConnectorError::new(
                ErrorCode::HealthCheckFailed,
                "Cannot start the Windows trusted HTTPS probe",
                false,
            )
            .with_detail("cause", error.to_string())
        })?;
    if !output.status.success() {
        return Err(ConnectorError::new(
            ErrorCode::HealthCheckFailed,
            "Nav Studio failed strict Windows HTTPS verification",
            true,
        )
        .with_detail("probe_exit_code", output.status.code().unwrap_or(-1)));
    }
    let status_text = String::from_utf8_lossy(&output.stdout);
    let status = status_text.trim().parse::<u16>().map_err(|_| {
        ConnectorError::new(
            ErrorCode::HealthCheckFailed,
            "Windows HTTPS probe returned no valid HTTP status",
            true,
        )
    })?;
    if (200..400).contains(&status) || status == 401 || status == 403 {
        Ok(HealthState::Healthy)
    } else {
        Err(ConnectorError::new(
            ErrorCode::HealthCheckFailed,
            format!("Nav Studio returned HTTP {status}"),
            true,
        ))
    }
}

fn connect_any(addresses: &[SocketAddr], timeout: Duration) -> Result<(TcpStream, SocketAddr)> {
    let mut last_error = None;
    for address in addresses {
        match TcpStream::connect_timeout(address, timeout) {
            Ok(stream) => return Ok((stream, *address)),
            Err(error) => last_error = Some(error),
        }
    }
    let error =
        last_error.unwrap_or_else(|| std::io::Error::new(ErrorKind::NotFound, "no address"));
    Err(ConnectorError::new(
        ErrorCode::EndpointUnreachable,
        "Cannot connect to the Nav Studio HTTPS port",
        true,
    )
    .with_detail("cause", error.to_string()))
}

fn certificate_metadata(der: &[u8]) -> (String, String, String, String) {
    match parse_x509_certificate(der) {
        Ok((_, certificate)) => (
            certificate.subject().to_string(),
            certificate.issuer().to_string(),
            certificate.validity().not_before.to_string(),
            certificate.validity().not_after.to_string(),
        ),
        Err(_) => (
            "unavailable".into(),
            "unavailable".into(),
            "unavailable".into(),
            "unavailable".into(),
        ),
    }
}

fn to_pem(der: &[u8]) -> String {
    let encoded = STANDARD.encode(der);
    let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
    for chunk in encoded.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(chunk).unwrap_or_default());
        pem.push('\n');
    }
    pem.push_str("-----END CERTIFICATE-----\n");
    pem
}
