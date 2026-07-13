use crate::EXIT_OK;
use serde_json::json;

pub fn run() -> u8 {
    let caps = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "operators": [
            {
                "name": "upscale",
                "description": "2x upscaling (Real-ESRGAN)",
                "models": ["upscale"],
                "batch": true,
                "params": {
                    "format": {"type": "enum", "values": ["png", "jpeg", "webp"], "default": "png"}
                }
            },
            {
                "name": "rembg",
                "description": "Background removal",
                "models": ["rembg", "rembg-hq"],
                "batch": true,
                "params": {
                    "quality": {"type": "enum", "values": ["fast", "high"], "default": "fast"},
                    "format": {"type": "enum", "values": ["png", "webp"], "default": "png"}
                }
            },
            {
                "name": "restore",
                "description": "Face restoration (GFPGAN)",
                "models": ["restore"],
                "batch": true,
                "params": {}
            },
            {
                "name": "segment",
                "description": "Object segmentation (SAM 2.1). JSON output includes iou and mask_pixels as confidence signals; low iou with tiny mask_pixels means the prompt missed the object - retry with a different point.",
                "models": ["sam"],
                "batch": false,
                "params": {
                    "point": {"type": "string", "format": "x,y pixels"},
                    "box": {"type": "string", "format": "x1,y1,x2,y2 pixels"}
                },
                "json_output": {"iou": "float 0-1", "mask_pixels": "int"}
            },
            {
                "name": "erase",
                "description": "Inpaint/remove masked region (LaMa). Compose with segment for click-to-erase: segment --point -> erase --mask.",
                "models": ["erase"],
                "batch": false,
                "params": {
                    "mask": {"type": "path", "description": "binary mask, white = erase"},
                    "dilate": {"type": "int", "default": 12}
                }
            }
        ],
        "conventions": {
            "batch_output": "-o is a file for single input, a directory for multiple; default is input-adjacent suffixed file",
            "stdin_stdout": "\"-\" as input reads stdin and writes stdout (single image)",
            "json_events": ["start", "done", "error", "summary"],
            "exit_codes": {"0": "all succeeded", "1": "partial failure", "2": "bad arguments", "3": "model missing", "4": "input unreadable"},
            "model_missing": "prints 'run: imagenie models pull <name>'; --yes downloads automatically"
        }
    });
    println!("{}", serde_json::to_string_pretty(&caps).unwrap());
    EXIT_OK
}
