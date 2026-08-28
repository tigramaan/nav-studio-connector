pub mod application;
pub mod cli;
pub mod domain;
pub mod network;
pub mod platform;

#[cfg(feature = "desktop")]
use application::{connector_status, diagnose, install_endpoint_trust, remove_owned_certificate};
#[cfg(feature = "desktop")]
use domain::{
    CallerMode, ConnectorError, ConnectorStatus, DiagnosticReport, DiscoveredRobot,
    EndpointInspection, HealthState, TrustOperation,
};
#[cfg(feature = "desktop")]
use network::{check_health, discover_robots, inspect_endpoint};

#[cfg(feature = "desktop")]
#[tauri::command]
fn discover(timeout_seconds: u64) -> Result<Vec<DiscoveredRobot>, ConnectorError> {
    discover_robots(timeout_seconds)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn inspect(url: String) -> Result<EndpointInspection, ConnectorError> {
    inspect_endpoint(&url, 8)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn install_trust(
    url: String,
    expected_fingerprint: String,
    identity_receipt: Option<String>,
    device_id: Option<String>,
    human_confirmed: bool,
) -> Result<TrustOperation, ConnectorError> {
    install_endpoint_trust(
        &url,
        Some(&expected_fingerprint),
        identity_receipt.as_deref(),
        device_id.as_deref(),
        human_confirmed,
        CallerMode::Human,
    )
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn remove_trust(fingerprint: String) -> Result<(), ConnectorError> {
    remove_owned_certificate(&fingerprint)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn health(url: String) -> Result<HealthState, ConnectorError> {
    check_health(&url, 8)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn status() -> Result<ConnectorStatus, ConnectorError> {
    connector_status()
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn diagnostics(url: Option<String>, timeout_seconds: u64) -> DiagnosticReport {
    diagnose(url.as_deref(), timeout_seconds)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn open_studio(url: String) -> Result<(), ConnectorError> {
    let parsed = domain::validate_https_url(&url)?;
    check_health(parsed.as_str(), 8)?;
    open::that(parsed.as_str()).map_err(|error| {
        domain::ConnectorError::new(
            domain::ErrorCode::InternalError,
            "Cannot open the default browser",
            true,
        )
        .with_detail("cause", error.to_string())
    })
}

#[cfg(feature = "desktop")]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            discover,
            inspect,
            install_trust,
            remove_trust,
            health,
            status,
            diagnostics,
            open_studio
        ])
        .run(tauri::generate_context!())
        .expect("error while running UMEC Nav Studio Connector");
}
