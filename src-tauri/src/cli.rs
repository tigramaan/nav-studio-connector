use crate::application::{
    connector_status, diagnose, install_endpoint_trust, remove_owned_certificate,
};
use crate::domain::{validate_https_url, CallerMode, ConnectorError, ErrorCode, Result};
use crate::network::{check_health, discover_robots, inspect_endpoint};
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::{json, Value};
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[command(
    name = "nav-studio-connector",
    version,
    about = "Discover and safely connect to UMEC Nav Studio"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },
    Discover {
        #[arg(long, default_value_t = 5)]
        timeout: u64,
        #[arg(long)]
        json: bool,
    },
    Inspect {
        #[arg(long)]
        url: String,
        #[arg(long)]
        json: bool,
    },
    Trust {
        #[command(subcommand)]
        action: TrustAction,
    },
    Status {
        #[arg(long)]
        json: bool,
    },
    Open {
        #[arg(long)]
        url: String,
        #[arg(long)]
        json: bool,
    },
    Diagnose {
        #[arg(long)]
        url: Option<String>,
        #[arg(long, default_value_t = 5)]
        timeout: u64,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum AgentAction {
    Describe {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum TrustAction {
    Plan {
        #[arg(long)]
        url: String,
        #[arg(long)]
        json: bool,
    },
    Install {
        #[arg(long)]
        url: String,
        #[arg(long)]
        expected_fingerprint: String,
        #[arg(long)]
        json: bool,
    },
    Remove {
        #[arg(long)]
        fingerprint: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Serialize)]
struct Envelope {
    schema_version: &'static str,
    ok: bool,
    operation: String,
    timestamp: String,
    data: Value,
    error: Option<ConnectorError>,
}

impl Envelope {
    fn success(operation: &str, data: impl Serialize) -> Self {
        Self {
            schema_version: "1.0",
            ok: true,
            operation: operation.into(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::to_value(data).unwrap_or(Value::Null),
            error: None,
        }
    }

    fn failure(operation: &str, error: ConnectorError) -> Self {
        Self {
            schema_version: "1.0",
            ok: false,
            operation: operation.into(),
            timestamp: Utc::now().to_rfc3339(),
            data: Value::Null,
            error: Some(error),
        }
    }
}

pub fn run_from<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let connector_error =
                ConnectorError::new(ErrorCode::InvalidInput, error.to_string(), false);
            print_envelope(&Envelope::failure("parse", connector_error));
            return 2;
        }
    };
    let (operation, result) = execute(cli.command);
    match result {
        Ok(data) => {
            print_envelope(&Envelope::success(&operation, data));
            0
        }
        Err(error) => {
            let exit_code = error.exit_code();
            print_envelope(&Envelope::failure(&operation, error));
            exit_code
        }
    }
}

fn execute(command: Command) -> (String, Result<Value>) {
    match command {
        Command::Agent {
            action: AgentAction::Describe { .. },
        } => (
            "agent.describe".into(),
            Ok(json!({
                "contract_version": "1.0",
                "executable": "nav-studio-connector",
                "discovery_service": crate::network::DISCOVERY_SERVICE,
                "commands": [
                    "agent describe --json", "discover --timeout 5 --json", "inspect --url <https-url> --json",
                    "trust plan --url <https-url> --json", "trust install --url <https-url> --expected-fingerprint <sha256> --json",
                    "trust remove --fingerprint <sha256> --json", "status --json", "open --url <https-url> --json",
                    "diagnose --url <https-url> --json"
                ],
                "trust_policy": "Never accept an unknown identity. Supply an independently verified SHA-256 fingerprint or a valid signed identity receipt."
            })),
        ),
        Command::Discover { timeout, .. } => (
            "discover".into(),
            discover_robots(timeout).and_then(to_value),
        ),
        Command::Inspect { url, .. } => (
            "inspect".into(),
            inspect_endpoint(&url, 8).and_then(to_value),
        ),
        Command::Trust {
            action: TrustAction::Plan { url, .. },
        } => {
            let result = inspect_endpoint(&url, 8).and_then(|inspection| to_value(json!({
                "studio_url": inspection.studio_url,
                "hostname": inspection.hostname,
                "fingerprint_sha256": inspection.fingerprint_sha256,
                "trust_target": if cfg!(windows) { "CurrentUser\\Root" } else { "/usr/local/share/ca-certificates/umec-nav-studio-<fingerprint-prefix>.crt" },
                "authorization_required": "expected_fingerprint_or_signed_receipt"
            })));
            ("trust.plan".into(), result)
        }
        Command::Trust {
            action:
                TrustAction::Install {
                    url,
                    expected_fingerprint,
                    ..
                },
        } => (
            "trust.install".into(),
            install_endpoint_trust(&url, Some(&expected_fingerprint), false, CallerMode::Agent)
                .and_then(to_value),
        ),
        Command::Trust {
            action: TrustAction::Remove { fingerprint, .. },
        } => (
            "trust.remove".into(),
            remove_owned_certificate(&fingerprint)
                .map(|_| json!({ "removed": true, "fingerprint_sha256": fingerprint })),
        ),
        Command::Status { .. } => ("status".into(), connector_status().and_then(to_value)),
        Command::Open { url, .. } => {
            let result = open_url(&url).map(|_| json!({ "opened": true, "studio_url": url }));
            ("open".into(), result)
        }
        Command::Diagnose { url, timeout, .. } => (
            "diagnose".into(),
            to_value(diagnose(url.as_deref(), timeout)),
        ),
    }
}

fn open_url(url: &str) -> Result<()> {
    let parsed = validate_https_url(url)?;
    check_health(parsed.as_str(), 8)?;
    open::that(parsed.as_str()).map_err(|error| {
        ConnectorError::new(
            ErrorCode::InternalError,
            "Cannot open the default browser",
            true,
        )
        .with_detail("cause", error.to_string())
    })
}

fn to_value<T: Serialize>(value: T) -> Result<Value> {
    serde_json::to_value(value).map_err(|error| {
        ConnectorError::new(
            ErrorCode::InternalError,
            "Cannot serialize command output",
            false,
        )
        .with_detail("cause", error.to_string())
    })
}

fn print_envelope(envelope: &Envelope) {
    println!(
        "{}",
        serde_json::to_string(envelope)
            .unwrap_or_else(|_| "{\"schema_version\":\"1.0\",\"ok\":false}".into())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_description_is_json_success() {
        let (operation, result) = execute(Command::Agent {
            action: AgentAction::Describe { json: true },
        });
        assert_eq!(operation, "agent.describe");
        assert!(result.unwrap()["commands"].is_array());
    }
}
