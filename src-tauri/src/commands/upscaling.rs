use std::path::Path;
use std::sync::OnceLock;
use tracing::info;

use crate::image::{model::UpscalingModel, processor::ModelProcessor, types::UpscalingParams};
use crate::utils::models_dir;

static UPSCALE_PROCESSOR: OnceLock<ModelProcessor<UpscalingModel>> = OnceLock::new();

#[tauri::command]
pub async fn init_upscaling() -> Result<(), String> {
    let models_dir = models_dir();
    UPSCALE_PROCESSOR.get_or_init(|| {
        let model_path = models_dir.join("image_upscaling.onnx");

        ModelProcessor::<UpscalingModel>::new(model_path.to_str().unwrap())
            .map_err(|e| e.to_string())
            .unwrap()
    });
    Ok(())
}

fn get_upscale_processor() -> &'static ModelProcessor<UpscalingModel> {
    UPSCALE_PROCESSOR.get().unwrap()
}

#[tauri::command]
pub async fn upscale_image(input_path: &str, output_dir: &str) -> Result<String, String> {
    info!("upscale_image was called with path: {}", input_path);

    let processor = get_upscale_processor();
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
pub async fn upscale_images(input_paths: Vec<String>, output_dir: &str) -> Result<(), String> {
    info!(
        "upscale_images was called with {} images",
        input_paths.len()
    );

    let processor = get_upscale_processor();
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
