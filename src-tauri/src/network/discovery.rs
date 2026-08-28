use crate::domain::{validate_timeout, ConnectorError, DiscoveredRobot, ErrorCode, Result};
use chrono::Utc;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

pub const DISCOVERY_SERVICE: &str = "_umec-nav._tcp.local.";

pub fn discover_robots(timeout_seconds: u64) -> Result<Vec<DiscoveredRobot>> {
    validate_timeout(timeout_seconds)?;
    let daemon = ServiceDaemon::new().map_err(|error| {
        ConnectorError::new(
            ErrorCode::EndpointUnreachable,
            "Cannot initialize local mDNS discovery",
            true,
        )
        .with_detail("cause", error.to_string())
    })?;
    let receiver = daemon.browse(DISCOVERY_SERVICE).map_err(|error| {
        ConnectorError::new(
            ErrorCode::EndpointUnreachable,
            "Cannot browse the Nav Studio DNS-SD service",
            true,
        )
        .with_detail("cause", error.to_string())
    })?;

    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    let mut robots = BTreeMap::new();
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(remaining.min(Duration::from_millis(350))) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                if let Ok(robot) = robot_from_service(&info) {
                    robots.insert(robot.robot_id.clone(), robot);
                }
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }
    let _ = daemon.stop_browse(DISCOVERY_SERVICE);
    let _ = daemon.shutdown();
    Ok(robots.into_values().collect())
}

fn robot_from_service(info: &ServiceInfo) -> Result<DiscoveredRobot> {
    let hostname = info
        .get_hostname()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if hostname.is_empty() || info.get_port() == 0 {
        return Err(ConnectorError::new(
            ErrorCode::InvalidServiceRecord,
            "DNS-SD record has no host or port",
            false,
        ));
    }
    let mut addresses: Vec<String> = info
        .get_addresses()
        .iter()
        .map(ToString::to_string)
        .collect();
    addresses.sort();
    addresses.dedup();
    if addresses.is_empty() {
        return Err(ConnectorError::new(
            ErrorCode::InvalidServiceRecord,
            "DNS-SD record has no resolved address",
            false,
        ));
    }

    let properties = info.get_properties();
    let supplied_id = properties
        .get_property_val_str("device_id")
        .unwrap_or_default()
        .trim();
    let robot_id = if valid_public_id(supplied_id) {
        supplied_id.to_string()
    } else {
        let digest = Sha256::digest(format!("{}|{}", info.get_fullname(), hostname));
        format!("derived-{}", &hex::encode(digest)[..12])
    };
    let path = properties
        .get_property_val_str("path")
        .filter(|value| value.starts_with('/') && value.len() <= 256)
        .unwrap_or("/");
    let port = info.get_port();
    let port_suffix = if port == 443 {
        String::new()
    } else {
        format!(":{port}")
    };
    let studio_url = format!("https://{hostname}{port_suffix}{path}");
    let short_id: String = robot_id.chars().take(8).collect();

    Ok(DiscoveredRobot {
        robot_id,
        display_name: format!("Робот {short_id}"),
        service_name: info.get_fullname().to_string(),
        hostname,
        addresses,
        port,
        studio_url,
        model: bounded_property(properties.get_property_val_str("model")),
        api_version: bounded_property(properties.get_property_val_str("api_version")),
        schema: bounded_property(properties.get_property_val_str("schema")),
        identity_method: "tls_fingerprint".to_string(),
        last_seen_at: Utc::now(),
    })
}

fn bounded_property(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .map(str::to_string)
}

fn valid_public_id(value: &str) -> bool {
    (6..=128).contains(&value.len())
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_public_ids() {
        assert!(valid_public_id("robot-ab12cd"));
        assert!(!valid_public_id("bad id"));
        assert!(!valid_public_id("x"));
    }
}
