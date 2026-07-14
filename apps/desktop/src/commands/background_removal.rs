use crate::utils::models_dir;
use std::path::Path;
use std::sync::OnceLock;
use tracing::info;

use imagenie_core::background::BackgroundRemover;
use imagenie_core::model::BackgroundRemovalModel;
use imagenie_core::types::BackgroundRemovalParams;
use imagenie_core::ImageOperator;

static BACKGROUND_REMOVER: OnceLock<BackgroundRemover<BackgroundRemovalModel>> = OnceLock::new();

#[tauri::command]
pub async fn init_background_removal() -> Result<(), String> {
    let models_dir = models_dir();
    BACKGROUND_REMOVER.get_or_init(|| {
        let model_path = models_dir.join("background_removal.onnx");
        let params = BackgroundRemovalParams {
            model_width: 1024,
            model_height: 1024,
            ..Default::default()
        };

        BackgroundRemover::new(model_path.to_str().unwrap(), params)
            .map_err(|e| e.to_string())
            .unwrap()
    });
    Ok(())
}

fn get_background_remover() -> &'static BackgroundRemover<BackgroundRemovalModel> {
    BACKGROUND_REMOVER.get().unwrap()
}

#[tauri::command]
pub async fn background_removal(input_path: &str, output_dir: &str) -> Result<String, String> {
    info!("background_removal was called with path: {}", input_path);

    let output = get_background_remover()
        .process(input_path)
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
    let output_path = Path::new(output_dir).join(format!("{}_removed.png", name));

    output.image.save(&output_path).map_err(|e| e.to_string())?;

    Ok(output_path.to_str().unwrap().to_string())
}
