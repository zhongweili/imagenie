use std::path::Path;
use std::sync::OnceLock;
use tracing::info;

use crate::image::{model::UpscalingModel, processor::ModelProcessor, types::UpscalingParams};

static UPSCALE_PROCESSOR: OnceLock<ModelProcessor<UpscalingModel>> = OnceLock::new();

fn get_upscale_processor() -> &'static ModelProcessor<UpscalingModel> {
    UPSCALE_PROCESSOR.get_or_init(|| {
        let model_path = Path::new("models").join("RealESRGAN_x2_fp16.onnx");

        ModelProcessor::<UpscalingModel>::new(model_path.to_str().unwrap())
            .map_err(|e| e.to_string())
            .unwrap()
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Helper function to get test fixtures path
    fn get_fixture_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("fixtures");
        path.push(filename);
        path
    }

    #[tokio::test]
    async fn test_upscale_single_image() {
        let test_image = get_fixture_path("test.png").to_str().unwrap().to_string();
        let test_dir = get_fixture_path("output").to_str().unwrap().to_string();
        if !Path::new(&test_dir).exists() {
            std::fs::create_dir_all(&test_dir).unwrap();
        }

        let result = upscale_image(&test_image, &test_dir).await;
        assert!(result.is_ok(), "Upscaling failed: {:?}", result.err());

        if let Ok(output) = result {
            let input_image = image::open(&test_image).unwrap();
            let output_path = std::fs::read_dir(&test_dir)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            let output_image = image::open(&output_path).unwrap();
            assert_eq!(output_image.width(), input_image.width() * 2);
            assert_eq!(output_image.height(), input_image.height() * 2);
            assert_eq!(output, output_path.to_str().unwrap().to_string());
        }

        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[tokio::test]
    async fn test_upscale_multiple_images() {
        let test_images = vec![
            get_fixture_path("test1.jpg").to_str().unwrap().to_string(),
            get_fixture_path("test2.jpg").to_str().unwrap().to_string(),
        ];

        let output_dir = get_fixture_path("output_multiple")
            .to_str()
            .unwrap()
            .to_string();
        if !Path::new(&output_dir).exists() {
            std::fs::create_dir_all(&output_dir).unwrap();
        }

        let result = upscale_images(test_images.clone(), &output_dir).await;
        assert!(result.is_ok(), "Batch upscaling failed: {:?}", result.err());

        if let Ok(()) = result {
            assert_eq!(test_images.len(), 2);
            assert_eq!(std::fs::read_dir(&output_dir).unwrap().count(), 2);
        }

        std::fs::remove_dir_all(&output_dir).unwrap();
    }

    #[tokio::test]
    async fn test_upscale_invalid_path() {
        let result = upscale_image("nonexistent.jpg", "output.png").await;
        assert!(result.is_err());
    }
}
