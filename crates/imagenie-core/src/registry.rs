use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::error::ImageProcessingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Payload {
    /// The download is the model file itself.
    File,
    /// The download is a zip; extract the listed inner paths.
    Zip { entries: &'static [&'static str] },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSpec {
    /// Registry name used in CLI (`imagenie models pull <name>`).
    pub name: &'static str,
    /// File name(s) on disk inside the models directory.
    pub files: &'static [&'static str],
    pub url: &'static str,
    /// sha256 of the downloaded payload (file or zip).
    pub sha256: &'static str,
    pub size_bytes: u64,
    pub payload: Payload,
}

pub const MODELS: &[ModelSpec] = &[
    ModelSpec {
        name: "rembg",
        files: &["background_removal.onnx"],
        url: "https://github.com/zhongweili/imagenie/releases/download/models-v0.1.0/background_removal.onnx",
        sha256: "b6e8497ba978a6f5fbb647e419d2696cd80df5a23cb6a8ea532021911bd76acb",
        size_bytes: 88_188_479,
        payload: Payload::File,
    },
    ModelSpec {
        name: "rembg-hq",
        files: &["birefnet_general_lite.onnx"],
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-general-bb_swin_v1_tiny-epoch_232.onnx",
        sha256: "5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333",
        size_bytes: 224_005_088,
        payload: Payload::File,
    },
    ModelSpec {
        name: "restore",
        files: &["face_restoration.onnx"],
        url: "https://github.com/zhongweili/imagenie/releases/download/models-v0.1.0/face_restoration.onnx",
        sha256: "9abba1f14737d8f83f66dab9abba28d7ef1ecf1f02bebb49f778f884820750a4",
        size_bytes: 340_256_689,
        payload: Payload::File,
    },
    ModelSpec {
        name: "upscale",
        files: &["image_upscaling.onnx"],
        url: "https://github.com/zhongweili/imagenie/releases/download/models-v0.1.0/image_upscaling.onnx",
        sha256: "22a5d72274bacd09a33dba691cc02f0819c3da218ed382e1e318e34a993a7ffe",
        size_bytes: 36_145_554,
        payload: Payload::File,
    },
    ModelSpec {
        name: "sam",
        files: &[
            "sam2.1_hiera_tiny.encoder.onnx",
            "sam2.1_hiera_tiny.decoder.onnx",
        ],
        url: "https://huggingface.co/vietanhdev/segment-anything-2.1-onnx-models/resolve/main/sam2.1_hiera_tiny_20260221.zip",
        sha256: "c602cb3f6fd297312a415f885f0df3f0eef9fbb334b069c9d30e828f2ae7c69a",
        size_bytes: 116_507_723,
        payload: Payload::Zip {
            entries: &[
                "sam2.1_tiny/sam2.1_hiera_tiny.encoder.onnx",
                "sam2.1_tiny/sam2.1_hiera_tiny.decoder.onnx",
            ],
        },
    },
    ModelSpec {
        name: "erase",
        files: &["lama_fp32.onnx"],
        url: "https://huggingface.co/Carve/LaMa-ONNX/resolve/main/lama_fp32.onnx",
        sha256: "1faef5301d78db7dda502fe59966957ec4b79dd64e16f03ed96913c7a4eb68d6",
        size_bytes: 208_044_816,
        payload: Payload::File,
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

/// Paths of the model's files inside the models directory.
pub fn model_paths(spec: &ModelSpec) -> Vec<PathBuf> {
    let dir = models_dir();
    spec.files.iter().map(|f| dir.join(f)).collect()
}

pub fn is_downloaded(spec: &ModelSpec) -> bool {
    model_paths(spec).iter().all(|p| p.exists())
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
) -> Result<Vec<PathBuf>, ImageProcessingError> {
    let dir = models_dir();
    fs::create_dir_all(&dir)?;

    let tmp_path = dir.join(format!("{}.download.partial", spec.name));

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

    let paths = model_paths(spec);
    match spec.payload {
        Payload::File => {
            fs::rename(&tmp_path, &paths[0])?;
        }
        Payload::Zip { entries } => {
            let file = fs::File::open(&tmp_path)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| ImageProcessingError::Processing(format!("bad zip: {e}")))?;
            for (entry, dest) in entries.iter().zip(paths.iter()) {
                let mut src = archive.by_name(entry).map_err(|e| {
                    ImageProcessingError::Processing(format!("zip entry {entry}: {e}"))
                })?;
                let mut out = fs::File::create(dest)?;
                std::io::copy(&mut src, &mut out)?;
            }
            fs::remove_file(&tmp_path).ok();
        }
    }

    Ok(paths)
}
