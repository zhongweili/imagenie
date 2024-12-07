use crate::utils::models_dir;
use image::DynamicImage;
use std::path::Path;
use std::sync::OnceLock;
use tracing::info;

use crate::image::{
    model::BackgroundRemovalModel, processor::ModelProcessor, types::BackgroundRemovalParams,
};

static BACKGROUND_REMOVAL_PROCESSOR: OnceLock<ModelProcessor<BackgroundRemovalModel>> =
    OnceLock::new();

#[tauri::command]
pub async fn init_background_removal() -> Result<(), String> {
    let models_dir = models_dir();
    BACKGROUND_REMOVAL_PROCESSOR.get_or_init(|| {
        let model_path = models_dir.join("background_removal.onnx");

        ModelProcessor::<BackgroundRemovalModel>::new(model_path.to_str().unwrap())
            .map_err(|e| e.to_string())
            .unwrap()
    });
    Ok(())
}

fn get_background_removal_processor() -> &'static ModelProcessor<BackgroundRemovalModel> {
    BACKGROUND_REMOVAL_PROCESSOR.get().unwrap()
}

#[tauri::command]
pub async fn background_removal(input_path: &str, output_dir: &str) -> Result<String, String> {
    info!("background_removal was called with path: {}", input_path);

    let processor = get_background_removal_processor();
    let params = BackgroundRemovalParams {
        model_width: 1024,
        model_height: 1024,
        ..Default::default()
    };

    // Get the mask from model processing
    let mask = processor
        .process_single(input_path, &params)
        .map_err(|e| e.to_string())?;

    // Load original image
    let original = image::open(input_path).map_err(|e| e.to_string())?;

    // Ensure original image and mask have same dimensions
    let mask = mask.resize_exact(
        original.width(),
        original.height(),
        image::imageops::FilterType::Triangle,
    );

    // Create final image by combining original colors with mask
    let mut final_image = image::RgbaImage::new(original.width(), original.height());
    let original_rgba = original.to_rgba8();
    let mask_rgba = mask.to_rgba8();

    for (x, y, pixel) in final_image.enumerate_pixels_mut() {
        let original_pixel = original_rgba.get_pixel(x, y);
        let mask_pixel = mask_rgba.get_pixel(x, y);

        *pixel = image::Rgba([
            original_pixel[0], // R from original
            original_pixel[1], // G from original
            original_pixel[2], // B from original
            mask_pixel[3],     // A from mask
        ]);
    }

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
    let output_path = Path::new(output_dir).join(format!("{}_removed.png", name));

    DynamicImage::ImageRgba8(final_image)
        .save(&output_path)
        .map_err(|e| e.to_string())?;

    Ok(output_path.to_str().unwrap().to_string())
}
