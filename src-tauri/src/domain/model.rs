use super::AuthorizationMethod;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredRobot {
    pub robot_id: String,
    pub display_name: String,
    pub service_name: String,
    pub hostname: String,
    pub addresses: Vec<String>,
    pub port: u16,
    pub studio_url: String,
    pub model: Option<String>,
    pub api_version: Option<String>,
    pub schema: Option<String>,
    pub identity_method: String,
    pub identity_receipt: Option<String>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdentityState {
    ReceiptVerified,
    HumanConfirmationRequired,
    Invalid,
    Mismatched,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustState {
    Trusted,
    Untrusted,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInspection {
    pub studio_url: String,
    pub hostname: String,
    pub resolved_address: String,
    pub certificate_pem: String,
    pub fingerprint_sha256: String,
    pub fingerprint_sha1: String,
    pub subject: String,
    pub issuer: String,
    pub not_before: String,
    pub not_after: String,
    pub identity_state: IdentityState,
    pub trust_state: TrustState,
    pub inspected_at: DateTime<Utc>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustOperation {
    pub operation_id: String,
    pub timestamp: DateTime<Utc>,
    pub platform: String,
    pub hostname: String,
    pub fingerprint_sha256: String,
    pub fingerprint_sha1: String,
    pub authorization_method: AuthorizationMethod,
    pub trust_target: String,
    pub installed: bool,
    pub health_state: HealthState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorStatus {
    pub app_version: String,
    pub platform: String,
    pub discovery_service: String,
    pub state_directory: String,
    pub receipt_verification_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub ok: bool,
    pub duration_ms: u64,
    pub code: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub schema_version: String,
    pub generated_at: DateTime<Utc>,
    pub platform: String,
    pub checks: Vec<DiagnosticCheck>,
    pub metadata: BTreeMap<String, String>,
}
