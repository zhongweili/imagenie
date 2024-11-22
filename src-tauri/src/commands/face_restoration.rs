use std::path::Path;
use std::sync::OnceLock;
use tauri::{path::BaseDirectory, AppHandle, Manager};
use tracing::info;

use crate::image::{
    model::FaceRestorationModel, processor::ModelProcessor, types::FaceRestorationParams,
};

static FACE_RESTORATION_PROCESSOR: OnceLock<ModelProcessor<FaceRestorationModel>> = OnceLock::new();

fn get_face_restoration_processor(
    app: &AppHandle,
) -> &'static ModelProcessor<FaceRestorationModel> {
    FACE_RESTORATION_PROCESSOR.get_or_init(|| {
        let model_path = app
            .path()
            .resolve("models/GFPGANv1.4.onnx", BaseDirectory::Resource)
            .expect("Failed to resolve resource path");

        ModelProcessor::<FaceRestorationModel>::new(model_path.to_str().unwrap())
            .map_err(|e| e.to_string())
            .unwrap()
    })
}

#[tauri::command]
pub async fn face_restoration(
    app: AppHandle,
    input_path: &str,
    output_dir: &str,
) -> Result<String, String> {
    info!("face_restoration was called with path: {}", input_path);

    let processor = get_face_restoration_processor(&app);
    let params = FaceRestorationParams {
        model_width: 512,
        model_height: 512,
        original_width: None,
        original_height: None,
        scaling_factor: None,
    };

    // Process image through the model
    let restored = processor
        .process_single(input_path, &params)
        .map_err(|e| e.to_string())?;

    // Save the result
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
    let output_path = Path::new(output_dir).join(format!("{}_restored.png", name));

    restored.save(&output_path).map_err(|e| e.to_string())?;

    Ok(output_path.to_str().unwrap().to_string())
}
