use serde_json::Value;
use std::process::Command;

#[test]
fn agent_description_matches_public_envelope_schema() {
    let output = Command::new(env!("CARGO_BIN_EXE_nav-studio-connector"))
        .args(["agent", "describe", "--json"])
        .output()
        .expect("connector binary must launch");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout must be JSON");
    let schema: Value = serde_json::from_str(include_str!(
        "../../specs/001-desktop-connector/contracts/agent-cli.schema.json"
    ))
    .expect("schema must be valid JSON");
    let validator = jsonschema::validator_for(&schema).expect("schema must compile");
    assert!(
        validator.is_valid(&value),
        "envelope did not match schema: {value}"
    );
    assert_eq!(value["operation"], "agent.describe");
    assert!(value["data"]["commands"]
        .as_array()
        .is_some_and(|commands| !commands.is_empty()));
}

#[test]
fn invalid_cli_input_is_json_and_exit_two() {
    let output = Command::new(env!("CARGO_BIN_EXE_nav-studio-connector"))
        .args(["inspect", "--url", "http://unsafe.local", "--json"])
        .output()
        .expect("connector binary must launch");
    assert_eq!(output.status.code(), Some(2));
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout must be JSON");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "INVALID_URL");
}
