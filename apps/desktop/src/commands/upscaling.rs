use std::path::Path;
use std::sync::OnceLock;
use tracing::info;

use crate::utils::models_dir;
use imagenie_core::{model::UpscalingModel, types::UpscalingParams, ImageOperator, ModelOperator};

static UPSCALER: OnceLock<ModelOperator<UpscalingModel>> = OnceLock::new();

#[tauri::command]
pub async fn init_upscaling() -> Result<(), String> {
    let models_dir = models_dir();
    UPSCALER.get_or_init(|| {
        let model_path = models_dir.join("image_upscaling.onnx");

        ModelOperator::<UpscalingModel>::new(
            model_path.to_str().unwrap(),
            UpscalingParams::default(),
        )
        .map_err(|e| e.to_string())
        .unwrap()
    });
    Ok(())
}

fn get_upscaler() -> &'static ModelOperator<UpscalingModel> {
    UPSCALER.get().unwrap()
}

#[tauri::command]
pub async fn upscale_image(input_path: &str, output_dir: &str) -> Result<String, String> {
    info!("upscale_image was called with path: {}", input_path);

    let image = get_upscaler()
        .process(input_path)
        .map(|output| output.image)
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

    let paths_clone = input_paths.clone();
    let outputs = get_upscaler().process_batch(&input_paths);

    for (i, output) in outputs.into_iter().enumerate() {
        let image = output.map_err(|e| e.to_string())?.image;
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
