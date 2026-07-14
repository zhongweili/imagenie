use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};

fn imagenie(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_imagenie"))
        .args(args)
        .output()
        .expect("run imagenie")
}

fn json_lines(output: &Output) -> Vec<Value> {
    String::from_utf8(output.stdout.clone())
        .expect("stdout is UTF-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("stdout line is JSON"))
        .collect()
}

#[test]
fn capabilities_exposes_five_versioned_operator_schemas() {
    let output = imagenie(&["capabilities", "--json"]);
    assert!(output.status.success());

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], "1.0");
    assert_eq!(value["operators"].as_array().unwrap().len(), 5);
    assert_eq!(value["event_contract"]["format"], "JSONL");
    for operator in value["operators"].as_array().unwrap() {
        assert_eq!(operator["input_schema"]["type"], "object");
    }
}

#[test]
fn semantic_usage_error_is_structured_and_terminates_with_summary() {
    let output = imagenie(&[
        "segment",
        "missing.png",
        "--json",
        "--request-id",
        "contract-test",
    ]);
    assert_eq!(output.status.code(), Some(2));

    let events = json_lines(&output);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["event"], "start");
    assert_eq!(events[1]["event"], "error");
    assert_eq!(events[1]["error"]["code"], "invalid_argument");
    assert_eq!(events[2]["event"], "summary");
    assert_eq!(events[2]["failed"], 1);
    assert!(events.iter().all(|event| {
        event["schema_version"] == "1.0" && event["request_id"] == "contract-test"
    }));
}

#[test]
fn json_mode_rejects_binary_stdout_before_loading_a_model() {
    let output = imagenie(&["upscale", "-", "--json", "--request-id", "pipe-test"]);
    assert_eq!(output.status.code(), Some(2));

    let events = json_lines(&output);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["error"]["code"], "pipe_conflict");
    assert_eq!(events[1]["event"], "summary");
}

#[test]
fn missing_model_has_stable_code_and_recovery() {
    let model_dir =
        std::env::temp_dir().join(format!("imagenie-contract-models-{}", std::process::id()));
    std::fs::create_dir_all(&model_dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_imagenie"))
        .args([
            "upscale",
            "missing.png",
            "--json",
            "--request-id",
            "model-test",
        ])
        .env("IMAGENIE_MODEL_DIR", &model_dir)
        .output()
        .expect("run imagenie");
    assert_eq!(output.status.code(), Some(3));

    let events = json_lines(&output);
    assert_eq!(events[0]["error"]["code"], "model_missing");
    assert!(
        events[0]["error"]["recovery"]
            .as_str()
            .unwrap()
            .contains("models pull")
    );
    assert_eq!(events.last().unwrap()["event"], "summary");

    std::fs::remove_dir_all(model_dir).ok();
}

#[test]
fn repository_contract_json_is_parseable() {
    let package = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "../../schemas/agent-events.schema.json",
        "../../benchmarks/manifest.json",
    ] {
        let bytes = std::fs::read(package.join(relative)).unwrap();
        serde_json::from_slice::<Value>(&bytes).unwrap();
    }
}
