use crate::{CommonArgs, OutputFormat};
use std::path::{Path, PathBuf};

impl OutputFormat {
    pub fn extension(self) -> &'static str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Webp => "webp",
        }
    }
}

/// Resolve the output path for one input.
/// - single input + -o file: use it
/// - multiple inputs + -o: treat as directory
/// - no -o: suffixed file next to the input (photo.upscaled.png)
pub fn resolve(input: &str, suffix: &str, ext: &str, common: &CommonArgs, multi: bool) -> PathBuf {
    let input_path = Path::new(input);
    let stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());

    match &common.output {
        Some(out) if !multi && !out.is_dir() => out.clone(),
        Some(out) => out.join(format!("{stem}.{suffix}.{ext}")),
        None => {
            let dir = input_path.parent().unwrap_or(Path::new("."));
            dir.join(format!("{stem}.{suffix}.{ext}"))
        }
    }
}

pub enum Collision {
    Write,
    Skip,
    Fail,
}

pub fn check_collision(path: &Path, common: &CommonArgs) -> Collision {
    if !path.exists() || common.force {
        Collision::Write
    } else if common.skip_existing {
        Collision::Skip
    } else {
        Collision::Fail
    }
}

/// Ensure -o directory exists when processing multiple inputs.
pub fn prepare_output_dir(common: &CommonArgs, multi: bool) -> std::io::Result<()> {
    if multi && let Some(out) = &common.output {
        std::fs::create_dir_all(out)?;
    }
    Ok(())
}
