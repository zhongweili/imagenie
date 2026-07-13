use serde::Serialize;
use std::io::Write;

/// Stable machine error codes for --json consumers.
pub mod codes {
    pub const INPUT_UNREADABLE: &str = "input_unreadable";
    pub const PROCESSING_FAILED: &str = "processing_failed";
    pub const OUTPUT_EXISTS: &str = "output_exists";
    pub const OUTPUT_WRITE_FAILED: &str = "output_write_failed";
}

#[derive(Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event<'a> {
    Start {
        op: &'a str,
        input: &'a str,
    },
    Done {
        op: &'a str,
        input: &'a str,
        output: &'a str,
        ms: u128,
        #[serde(skip_serializing_if = "Option::is_none")]
        iou: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mask_pixels: Option<u64>,
    },
    Error {
        op: &'a str,
        input: &'a str,
        code: &'a str,
        message: &'a str,
    },
    Summary {
        op: &'a str,
        total: usize,
        succeeded: usize,
        failed: usize,
        skipped: usize,
    },
}

pub struct Emitter {
    json: bool,
}

impl Emitter {
    pub fn new(json: bool) -> Self {
        Self { json }
    }

    pub fn emit(&self, event: &Event) {
        if self.json {
            let mut stdout = std::io::stdout().lock();
            serde_json::to_writer(&mut stdout, event).ok();
            stdout.write_all(b"\n").ok();
            stdout.flush().ok();
        } else {
            match event {
                Event::Done {
                    input,
                    output,
                    ms,
                    iou,
                    mask_pixels,
                    ..
                } => {
                    let extra = match (iou, mask_pixels) {
                        (Some(iou), Some(px)) => format!(" (iou {iou:.3}, {px} px)"),
                        _ => String::new(),
                    };
                    eprintln!("{input} -> {output} ({ms}ms){extra}");
                }
                Event::Error { input, message, .. } => {
                    eprintln!("error: {input}: {message}");
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
