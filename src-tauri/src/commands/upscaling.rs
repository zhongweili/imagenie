use std::path::Path;
use std::sync::OnceLock;
use tracing::info;

use crate::image::{model::UpscalingModel, processor::ModelProcessor, types::UpscalingParams};

static UPSCALE_PROCESSOR: OnceLock<ModelProcessor<UpscalingModel>> = OnceLock::new();

fn get_upscale_processor() -> &'static ModelProcessor<UpscalingModel> {
    UPSCALE_PROCESSOR.get_or_init(|| {
        let model_path = Path::new("models").join("RealESRGAN_x2_fp16.onnx");

        ModelProcessor::<UpscalingModel>::new(&model_path.to_str().unwrap())
            .map_err(|e| e.to_string())
            .unwrap()
    })
}

#[tauri::command]
pub fn upscale_image(input_path: &str, output_path: &str) -> Result<(), String> {
    info!("upscale_image was called with path: {}", input_path);

    let processor = get_upscale_processor();
    let params = UpscalingParams {};

    let image = processor
        .process_single(input_path, &params)
        .map_err(|e| e.to_string())?;
    image.save(output_path).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn upscale_images(input_paths: Vec<String>, output_dir: &str) -> Result<(), String> {
    info!(
        "upscale_images was called with {} images",
        input_paths.len()
    );

    let processor = get_upscale_processor();

    let params = UpscalingParams {};

    let images = processor
        .process_batch(input_paths, &params)
        .map_err(|e| e.to_string())?;

    for (i, image) in images.iter().enumerate() {
        let output_path = Path::new(output_dir).join(format!("output_{}.png", i));
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

    #[test]
    fn test_upscale_single_image() {
        let test_image = get_fixture_path("test.png").to_str().unwrap().to_string();
        let test_output = get_fixture_path("output.png").to_str().unwrap().to_string();

        let result = upscale_image(&test_image, &test_output);
        assert!(result.is_ok(), "Upscaling failed: {:?}", result.err());

        if let Ok(()) = result {
            let input_image = image::open(&test_image).unwrap();
            let output_image = image::open(&test_output).unwrap();
            assert_eq!(output_image.width(), input_image.width() * 2);
            assert_eq!(output_image.height(), input_image.height() * 2);
        }
    }

    #[test]
    fn test_upscale_multiple_images() {
        let test_images = vec![
            get_fixture_path("test1.jpg").to_str().unwrap().to_string(),
            get_fixture_path("test2.jpg").to_str().unwrap().to_string(),
        ];

        let output_dir = get_fixture_path("output").to_str().unwrap().to_string();
        let output_path = Path::new(&output_dir);
        if !output_path.exists() {
            std::fs::create_dir_all(&output_dir).unwrap();
        }

        let result = upscale_images(test_images.clone(), &output_dir);
        assert!(result.is_ok(), "Batch upscaling failed: {:?}", result.err());

        if let Ok(()) = result {
            assert_eq!(test_images.len(), 2);
            assert_eq!(std::fs::read_dir(output_dir).unwrap().count(), 2);
        }
    }

    #[test]
    fn test_upscale_invalid_path() {
        let result = upscale_image("nonexistent.jpg", "output.png");
        assert!(result.is_err());
    }
}
