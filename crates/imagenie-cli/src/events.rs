use imagenie_core::ProcessMetrics;
use imagenie_core::registry::ModelSpec;
use serde::Serialize;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

pub const SCHEMA_VERSION: &str = "1.0";

/// Stable machine error codes for --json consumers.
pub mod codes {
    pub const INPUT_UNREADABLE: &str = "input_unreadable";
    pub const INVALID_ARGUMENT: &str = "invalid_argument";
    pub const MODEL_LOAD_FAILED: &str = "model_load_failed";
    pub const MODEL_MISSING: &str = "model_missing";
    pub const OUTPUT_DIR_FAILED: &str = "output_dir_failed";
    pub const OUTPUT_EXISTS: &str = "output_exists";
    pub const OUTPUT_WRITE_FAILED: &str = "output_write_failed";
    pub const PIPE_CONFLICT: &str = "pipe_conflict";
    pub const PROCESSING_FAILED: &str = "processing_failed";

    pub fn recovery(code: &str) -> &'static str {
        match code {
            INPUT_UNREADABLE => "Check that the input exists and is a supported readable image.",
            INVALID_ARGUMENT => {
                "Correct the arguments using `imagenie <command> --help` and retry."
            }
            MODEL_LOAD_FAILED => {
                "Re-pull the model and verify that the current platform is supported."
            }
            MODEL_MISSING => {
                "Run the suggested `imagenie models pull <name>` command or retry with --yes."
            }
            OUTPUT_DIR_FAILED => "Choose a writable output directory and retry.",
            OUTPUT_EXISTS => {
                "Retry with --force to overwrite or --skip-existing to keep the existing file."
            }
            OUTPUT_WRITE_FAILED => "Check output permissions and free disk space, then retry.",
            PIPE_CONFLICT => {
                "Use a file output for structured events, or omit --json for binary stdout."
            }
            _ => "Inspect the error message, correct the input or environment, and retry.",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ModelIdentity {
    pub name: String,
    pub artifact_sha256: String,
}

impl From<&ModelSpec> for ModelIdentity {
    fn from(spec: &ModelSpec) -> Self {
        Self {
            name: spec.name.to_string(),
            artifact_sha256: spec.sha256.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails<'a> {
    pub code: &'a str,
    pub message: &'a str,
    pub recovery: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event<'a> {
    Start {
        input_index: usize,
        input: &'a str,
    },
    Done {
        input_index: usize,
        input: &'a str,
        output: &'a str,
        duration_ms: u128,
        metrics: ProcessMetrics,
        warnings: &'a [String],
    },
    Skipped {
        input_index: usize,
        input: &'a str,
        output: &'a str,
        reason: &'a str,
    },
    Error {
        input_index: usize,
        input: &'a str,
        error: ErrorDetails<'a>,
    },
    Summary {
        duration_ms: u128,
        total: usize,
        succeeded: usize,
        failed: usize,
        skipped: usize,
    },
}

impl<'a> Event<'a> {
    pub fn error(input_index: usize, input: &'a str, code: &'a str, message: &'a str) -> Self {
        Self::Error {
            input_index,
            input,
            error: ErrorDetails {
                code,
                message,
                recovery: codes::recovery(code),
            },
        }
    }
}

#[derive(Serialize)]
struct WireEvent<'a> {
    schema_version: &'static str,
    request_id: &'a str,
    operator: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<&'a ModelIdentity>,
    #[serde(flatten)]
    body: &'a Event<'a>,
}

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn generated_request_id() -> String {
    let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("img-{}-{sequence}", std::process::id())
}

pub struct Emitter {
    json: bool,
    request_id: String,
    operator: String,
    model: Option<ModelIdentity>,
}

impl Emitter {
    pub fn new(
        json: bool,
        request_id: Option<&str>,
        operator: &str,
        model: Option<&ModelSpec>,
    ) -> Self {
        Self {
            json,
            request_id: request_id
                .map(str::to_owned)
                .unwrap_or_else(generated_request_id),
            operator: operator.to_string(),
            model: model.map(ModelIdentity::from),
        }
    }

    fn serialize(&self, event: &Event<'_>) -> serde_json::Result<String> {
        serde_json::to_string(&WireEvent {
            schema_version: SCHEMA_VERSION,
            request_id: &self.request_id,
            operator: &self.operator,
            model: self.model.as_ref(),
            body: event,
        })
    }

    pub fn emit(&self, event: &Event<'_>) {
        if self.json {
            let mut stdout = std::io::stdout().lock();
            if let Ok(line) = self.serialize(event) {
                stdout.write_all(line.as_bytes()).ok();
                stdout.write_all(b"\n").ok();
                stdout.flush().ok();
            }
        } else {
            match event {
                Event::Done {
                    input,
                    output,
                    duration_ms,
                    metrics,
                    ..
                } => {
                    let extra = match (metrics.iou, metrics.mask_pixels) {
                        (Some(iou), Some(px)) => format!(" (iou {iou:.3}, {px} px)"),
                        _ => String::new(),
                    };
                    eprintln!("{input} -> {output} ({duration_ms}ms){extra}");
                }
                Event::Skipped { input, output, .. } => {
                    eprintln!("{input} -> {output} (skipped)");
                }
                Event::Error { input, error, .. } => {
                    eprintln!("error: {input}: {}", error.message);
                }
                Event::Summary {
                    total,
                    succeeded,
                    failed,
                    skipped,
                    ..
                } => {
                    if *total > 1 || *failed > 0 || *skipped > 0 {
                        eprintln!(
                            "{succeeded}/{total} succeeded, {failed} failed, {skipped} skipped"
                        );
                    }
                }
                Event::Start { .. } => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_event_has_stable_envelope_and_nested_error() {
        let emitter = Emitter::new(true, Some("request-42"), "segment", None);
        let event = Event::error(0, "missing.png", codes::INPUT_UNREADABLE, "file not found");
        let value: serde_json::Value =
            serde_json::from_str(&emitter.serialize(&event).unwrap()).unwrap();

        assert_eq!(value["schema_version"], SCHEMA_VERSION);
        assert_eq!(value["request_id"], "request-42");
        assert_eq!(value["operator"], "segment");
        assert_eq!(value["event"], "error");
        assert_eq!(value["input_index"], 0);
        assert_eq!(value["error"]["code"], codes::INPUT_UNREADABLE);
        assert!(
            value["error"]["recovery"]
                .as_str()
                .unwrap()
                .contains("input")
        );
    }

    #[test]
    fn done_event_always_contains_metrics_and_warnings() {
        let emitter = Emitter::new(true, Some("request-7"), "upscale", None);
        let event = Event::Done {
            input_index: 1,
            input: "in.png",
            output: "out.png",
            duration_ms: 12,
            metrics: ProcessMetrics::default(),
            warnings: &[],
        };
        let value: serde_json::Value =
            serde_json::from_str(&emitter.serialize(&event).unwrap()).unwrap();

        assert_eq!(value["metrics"], serde_json::json!({}));
        assert_eq!(value["warnings"], serde_json::json!([]));
    }
}
