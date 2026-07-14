use std::path::Path;
use std::sync::OnceLock;
use tracing::info;

use crate::utils::models_dir;
use imagenie_core::{
    model::FaceRestorationModel, types::FaceRestorationParams, ImageOperator, ModelOperator,
};

static FACE_RESTORER: OnceLock<ModelOperator<FaceRestorationModel>> = OnceLock::new();

#[tauri::command]
pub async fn init_face_restoration() -> Result<(), String> {
    let models_dir = models_dir();
    FACE_RESTORER.get_or_init(|| {
        let model_path = models_dir.join("face_restoration.onnx");

        ModelOperator::<FaceRestorationModel>::new(
            model_path.to_str().unwrap(),
            FaceRestorationParams::default(),
        )
        .map_err(|e| e.to_string())
        .unwrap()
    });
    Ok(())
}

fn get_face_restorer() -> &'static ModelOperator<FaceRestorationModel> {
    FACE_RESTORER.get().unwrap()
}

#[tauri::command]
pub async fn face_restoration(input_path: &str, output_dir: &str) -> Result<String, String> {
    info!("face_restoration was called with path: {}", input_path);

    let restored = get_face_restorer()
        .process(input_path)
        .map(|output| output.image)
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
