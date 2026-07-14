use crate::EXIT_OK;
use crate::events::SCHEMA_VERSION;
use serde_json::{Value, json};

pub fn document() -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "cli_version": env!("CARGO_PKG_VERSION"),
        "operators": [
            {
                "name": "upscale",
                "description": "Locally upscale one or more images by 2x. Use for low-resolution raster images; do not use when exact original pixels must be preserved.",
                "models": ["upscale"],
                "batch": true,
                "input_schema": image_batch_schema(json!({
                    "format": {"type": "string", "enum": ["png", "jpeg", "webp"], "default": "png"}
                }))
            },
            {
                "name": "rembg",
                "description": "Remove image backgrounds and write an alpha-capable PNG or WebP. Use high quality for fine edges when extra latency is acceptable.",
                "models": ["rembg", "rembg-hq"],
                "batch": true,
                "input_schema": image_batch_schema(json!({
                    "quality": {"type": "string", "enum": ["fast", "high"], "default": "fast"},
                    "format": {"type": "string", "enum": ["png", "webp"], "default": "png"}
                }))
            },
            {
                "name": "restore",
                "description": "Restore faces in one or more local images with GFPGAN. Use for degraded faces; inspect the output when identity preservation is critical.",
                "models": ["restore"],
                "batch": true,
                "input_schema": image_batch_schema(json!({}))
            },
            {
                "name": "segment",
                "description": "Create an object mask from exactly one point or box prompt. Inspect iou and mask_pixels; retry with a better prompt when confidence is low or the mask is tiny.",
                "models": ["sam"],
                "batch": false,
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "input": {"type": "string", "description": "Readable local image path"},
                        "point": {"type": "string", "pattern": "^-?[0-9]+(?:\\.[0-9]+)?,-?[0-9]+(?:\\.[0-9]+)?$"},
                        "box": {"type": "string", "description": "x1,y1,x2,y2 in pixels"},
                        "output": {"type": "string"}
                    },
                    "required": ["input"],
                    "oneOf": [{"required": ["point"]}, {"required": ["box"]}],
                    "additionalProperties": false
                },
                "result_metrics": {
                    "iou": {"type": "number", "minimum": 0, "maximum": 1},
                    "mask_pixels": {"type": "integer", "minimum": 0}
                }
            },
            {
                "name": "erase",
                "description": "Inpaint a white mask region with LaMa. Compose with segment for click-to-erase; review the result around mask boundaries.",
                "models": ["erase"],
                "batch": false,
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "input": {"type": "string", "description": "Readable local image path"},
                        "mask": {"type": "string", "description": "Binary mask path; white pixels are erased"},
                        "dilate": {"type": "integer", "minimum": 0, "default": 12},
                        "output": {"type": "string"}
                    },
                    "required": ["input", "mask"],
                    "additionalProperties": false
                }
            }
        ],
        "event_contract": {
            "format": "JSONL",
            "schema": "schemas/agent-events.schema.json",
            "events": ["start", "done", "skipped", "error", "summary"],
            "required_envelope": ["schema_version", "request_id", "operator", "event"],
            "batch_correlation": "input_index",
            "stdout": "JSONL only when --json is set; binary pipe output cannot be combined with --json",
            "stderr": "human diagnostics and progress"
        },
        "exit_codes": {
            "0": "all succeeded or explicitly skipped",
            "1": "partial or processing failure",
            "2": "invalid invocation",
            "3": "model unavailable",
            "4": "all requested inputs unreadable"
        }
    })
}

fn image_batch_schema(extra: Value) -> Value {
    let mut properties = json!({
        "inputs": {"type": "array", "items": {"type": "string"}, "minItems": 1},
        "output": {"type": "string", "description": "File for one input or directory for multiple inputs"},
        "jobs": {"type": "integer", "minimum": 1},
        "force": {"type": "boolean", "default": false},
        "skip_existing": {"type": "boolean", "default": false},
        "request_id": {"type": "string"}
    });
    if let (Some(base), Some(extra)) = (properties.as_object_mut(), extra.as_object()) {
        base.extend(extra.clone());
    }
    json!({
        "type": "object",
        "properties": properties,
        "required": ["inputs"],
        "additionalProperties": false
    })
}

pub fn run() -> u8 {
    println!("{}", serde_json::to_string_pretty(&document()).unwrap());
    EXIT_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_are_versioned_and_schema_first() {
        let caps = document();
        assert_eq!(caps["schema_version"], SCHEMA_VERSION);
        assert_eq!(caps["operators"].as_array().unwrap().len(), 5);
        for operator in caps["operators"].as_array().unwrap() {
            assert_eq!(operator["input_schema"]["type"], "object");
            assert!(operator["description"].as_str().unwrap().len() > 40);
        }
        assert_eq!(caps["event_contract"]["format"], "JSONL");
    }
}
