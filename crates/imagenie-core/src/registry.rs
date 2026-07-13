use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::error::ImageProcessingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSpec {
    /// Registry name used in CLI (`imagenie models pull <name>`).
    pub name: &'static str,
    /// File name on disk inside the models directory.
    pub file_name: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub size_bytes: u64,
}

pub const MODELS: &[ModelSpec] = &[
    ModelSpec {
        name: "rembg",
        file_name: "background_removal.onnx",
        url: "https://github.com/zhongweili/imagenie/releases/download/models-v0.1.0/background_removal.onnx",
        sha256: "b6e8497ba978a6f5fbb647e419d2696cd80df5a23cb6a8ea532021911bd76acb",
        size_bytes: 88_188_479,
    },
    ModelSpec {
        name: "restore",
        file_name: "face_restoration.onnx",
        url: "https://github.com/zhongweili/imagenie/releases/download/models-v0.1.0/face_restoration.onnx",
        sha256: "9abba1f14737d8f83f66dab9abba28d7ef1ecf1f02bebb49f778f884820750a4",
        size_bytes: 340_256_689,
    },
    ModelSpec {
        name: "upscale",
        file_name: "image_upscaling.onnx",
        url: "https://github.com/zhongweili/imagenie/releases/download/models-v0.1.0/image_upscaling.onnx",
        sha256: "22a5d72274bacd09a33dba691cc02f0819c3da218ed382e1e318e34a993a7ffe",
        size_bytes: 36_145_554,
    },
];

pub fn find(name: &str) -> Option<&'static ModelSpec> {
    MODELS.iter().find(|m| m.name == name)
}

/// Models directory shared with the desktop app. Override with `IMAGENIE_MODEL_DIR`.
pub fn models_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("IMAGENIE_MODEL_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".imagenie")
        .join("models")
}

pub fn model_path(spec: &ModelSpec) -> PathBuf {
    models_dir().join(spec.file_name)
}

pub fn is_downloaded(spec: &ModelSpec) -> bool {
    model_path(spec).exists()
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub model_name: String,
    pub downloaded: u64,
    pub total: u64,
}

/// Download a model to the models directory, verifying its sha256.
/// Blocking; call from a worker thread in async contexts.
pub fn download(
    spec: &ModelSpec,
    on_progress: impl Fn(Progress),
) -> Result<PathBuf, ImageProcessingError> {
    let dir = models_dir();
    fs::create_dir_all(&dir)?;

    let final_path = dir.join(spec.file_name);
    let tmp_path = dir.join(format!("{}.partial", spec.file_name));

    let mut response = reqwest::blocking::get(spec.url)
        .and_then(|r| r.error_for_status())
        .map_err(|e| ImageProcessingError::Processing(format!("download failed: {e}")))?;

    let total = response.content_length().unwrap_or(spec.size_bytes);
    let mut file = fs::File::create(&tmp_path)?;
    let mut hasher = Sha256::new();
    let mut downloaded = 0u64;
    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = response
            .read(&mut buf)
            .map_err(|e| ImageProcessingError::Processing(format!("download failed: {e}")))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        hasher.update(&buf[..n]);
        downloaded += n as u64;
        on_progress(Progress {
            model_name: spec.name.to_string(),
            downloaded,
            total,
        });
    }
    drop(file);

    let digest = format!("{:x}", hasher.finalize());
    if digest != spec.sha256 {
        fs::remove_file(&tmp_path).ok();
        return Err(ImageProcessingError::Processing(format!(
            "sha256 mismatch for {}: expected {}, got {digest}",
            spec.name, spec.sha256
        )));
    }

    fs::rename(&tmp_path, &final_path)?;
    Ok(final_path)
}
