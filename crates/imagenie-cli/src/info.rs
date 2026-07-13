use crate::{EXIT_INPUT_UNREADABLE, EXIT_OK, EXIT_PARTIAL};
use serde::Serialize;

#[derive(Serialize)]
struct ImageInfo {
    path: String,
    width: u32,
    height: u32,
    format: String,
    size_bytes: u64,
}

pub fn run(inputs: &[String], json: bool) -> u8 {
    let mut failed = 0;
    for input in inputs {
        match probe(input) {
            Ok(info) => {
                if json {
                    println!("{}", serde_json::to_string(&info).unwrap());
                } else {
                    println!(
                        "{}: {}x{} {} ({} bytes)",
                        info.path, info.width, info.height, info.format, info.size_bytes
                    );
                }
            }
            Err(e) => {
                eprintln!("error: {input}: {e}");
                failed += 1;
            }
        }
    }
    if failed == inputs.len() {
        EXIT_INPUT_UNREADABLE
    } else if failed > 0 {
        EXIT_PARTIAL
    } else {
        EXIT_OK
    }
}

fn probe(path: &str) -> anyhow::Result<ImageInfo> {
    let meta = std::fs::metadata(path)?;
    let reader = image::ImageReader::open(path)?.with_guessed_format()?;
    let format = reader
        .format()
        .map(|f| format!("{f:?}").to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());
    let (width, height) = reader.into_dimensions()?;
    Ok(ImageInfo {
        path: path.to_string(),
        width,
        height,
        format,
        size_bytes: meta.len(),
    })
}
