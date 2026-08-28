use crate::domain::{DiagnosticCheck, DiagnosticReport};
use crate::network::{check_health, discover_robots, inspect_endpoint};
use crate::platform;
use chrono::Utc;
use std::collections::BTreeMap;
use std::time::Instant;

pub fn diagnose(url: Option<&str>, timeout_seconds: u64) -> DiagnosticReport {
    let mut checks = Vec::new();
    let discovery_started = Instant::now();
    match discover_robots(timeout_seconds) {
        Ok(robots) => checks.push(DiagnosticCheck {
            name: "mdns_discovery".into(),
            ok: true,
            duration_ms: elapsed_ms(discovery_started),
            code: None,
            summary: format!("{} candidate(s) discovered", robots.len()),
        }),
        Err(error) => checks.push(DiagnosticCheck {
            name: "mdns_discovery".into(),
            ok: false,
            duration_ms: elapsed_ms(discovery_started),
            code: Some(error.code.as_str().into()),
            summary: error.message,
        }),
    }

    if let Some(url) = url {
        let inspection_started = Instant::now();
        match inspect_endpoint(url, timeout_seconds) {
            Ok(inspection) => checks.push(DiagnosticCheck {
                name: "tls_inspection".into(),
                ok: true,
                duration_ms: elapsed_ms(inspection_started),
                code: None,
                summary: format!("TLS certificate SHA-256 {}", inspection.fingerprint_sha256),
            }),
            Err(error) => checks.push(DiagnosticCheck {
                name: "tls_inspection".into(),
                ok: false,
                duration_ms: elapsed_ms(inspection_started),
                code: Some(error.code.as_str().into()),
                summary: error.message,
            }),
        }
        let health_started = Instant::now();
        match check_health(url, timeout_seconds) {
            Ok(_) => checks.push(DiagnosticCheck {
                name: "https_health".into(),
                ok: true,
                duration_ms: elapsed_ms(health_started),
                code: None,
                summary: "Trusted HTTPS endpoint is reachable".into(),
            }),
            Err(error) => checks.push(DiagnosticCheck {
                name: "https_health".into(),
                ok: false,
                duration_ms: elapsed_ms(health_started),
                code: Some(error.code.as_str().into()),
                summary: error.message,
            }),
        }
    }

    let mut metadata = BTreeMap::new();
    metadata.insert("app_version".into(), env!("CARGO_PKG_VERSION").into());
    metadata.insert(
        "discovery_service".into(),
        crate::network::DISCOVERY_SERVICE.into(),
    );
    DiagnosticReport {
        schema_version: "1.0".into(),
        generated_at: Utc::now(),
        platform: platform::platform_name().into(),
        checks,
        metadata,
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}
