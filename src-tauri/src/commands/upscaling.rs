use std::path::Path;
use std::sync::OnceLock;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tracing::info;

use crate::image::{model::UpscalingModel, processor::ModelProcessor, types::UpscalingParams};

static UPSCALE_PROCESSOR: OnceLock<ModelProcessor<UpscalingModel>> = OnceLock::new();

fn get_upscale_processor(app: &AppHandle) -> &'static ModelProcessor<UpscalingModel> {
    UPSCALE_PROCESSOR.get_or_init(|| {
        let model_path = app
            .path()
            .resolve("models/RealESRGAN_x2_fp16.onnx", BaseDirectory::Resource)
            .expect("Failed to resolve resource path");

        info!("Loading model from path: {:?}", model_path);

        ModelProcessor::<UpscalingModel>::new(model_path.to_str().unwrap())
            .map_err(|e| e.to_string())
            .unwrap()
    })
}

#[tauri::command]
pub async fn upscale_image(
    app: AppHandle,
    input_path: &str,
    output_dir: &str,
) -> Result<String, String> {
    info!("upscale_image was called with path: {}", input_path);

    let processor = get_upscale_processor(&app);
    let params = UpscalingParams {};

    let image = processor
        .process_single(input_path, &params)
        .map_err(|e| e.to_string())?;

    let name = Path::new(input_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let name = name
        .strip_suffix(".png")
        .or_else(|| name.strip_suffix(".jpg"))
        .or_else(|| name.strip_suffix(".jpeg"))
        .unwrap_or(&name);
    let output_path = Path::new(output_dir).join(format!("{}_upscaled.png", name));
    image.save(&output_path).map_err(|e| e.to_string())?;

    Ok(output_path.to_str().unwrap().to_string())
}

#[tauri::command]
pub async fn upscale_images(
    app: AppHandle,
    input_paths: Vec<String>,
    output_dir: &str,
) -> Result<(), String> {
    info!(
        "upscale_images was called with {} images",
        input_paths.len()
    );

    let processor = get_upscale_processor(&app);
    let params = UpscalingParams {};

    let paths_clone = input_paths.clone();
    let images = processor
        .process_batch(input_paths, &params)
        .map_err(|e| e.to_string())?;

    for (i, image) in images.iter().enumerate() {
        let name = Path::new(&paths_clone[i])
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let name = name
            .strip_suffix(".png")
            .or_else(|| name.strip_suffix(".jpg"))
            .or_else(|| name.strip_suffix(".jpeg"))
            .unwrap_or(&name);
        let output_path = Path::new(output_dir).join(format!("{}_upscaled.png", name));
        image.save(&output_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
