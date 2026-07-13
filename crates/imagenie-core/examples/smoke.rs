use imagenie_core::model::{BackgroundRemovalModel, FaceRestorationModel, UpscalingModel};
use imagenie_core::processor::ModelProcessor;
use imagenie_core::registry;
use imagenie_core::types::{BackgroundRemovalParams, FaceRestorationParams, UpscalingParams};
use std::time::Instant;

fn main() {
    let input = std::env::args().nth(1).expect("usage: smoke <image>");
    let models = registry::models_dir();
    let out_dir = std::env::temp_dir().join("imagenie-smoke");
    std::fs::create_dir_all(&out_dir).unwrap();

    let t = Instant::now();
    let upscaler = ModelProcessor::<UpscalingModel>::new(
        models.join("image_upscaling.onnx").to_str().unwrap(),
    )
    .unwrap();
    let img = upscaler
        .process_single(&input, &UpscalingParams::default())
        .unwrap();
    img.save(out_dir.join("upscaled.png")).unwrap();
    println!(
        "upscale OK ({}x{}) in {:?}",
        img.width(),
        img.height(),
        t.elapsed()
    );

    let t = Instant::now();
    let rembg = ModelProcessor::<BackgroundRemovalModel>::new(
        models.join("background_removal.onnx").to_str().unwrap(),
    )
    .unwrap();
    let params = BackgroundRemovalParams {
        model_width: 1024,
        model_height: 1024,
        ..Default::default()
    };
    let mask = rembg.process_single(&input, &params).unwrap();
    mask.save(out_dir.join("mask.png")).unwrap();
    println!(
        "rembg OK ({}x{}) in {:?}",
        mask.width(),
        mask.height(),
        t.elapsed()
    );

    let t = Instant::now();
    let restore = ModelProcessor::<FaceRestorationModel>::new(
        models.join("face_restoration.onnx").to_str().unwrap(),
    )
    .unwrap();
    let img = restore
        .process_single(&input, &FaceRestorationParams::default())
        .unwrap();
    img.save(out_dir.join("restored.png")).unwrap();
    println!(
        "restore OK ({}x{}) in {:?}",
        img.width(),
        img.height(),
        t.elapsed()
    );

    println!("outputs in {}", out_dir.display());
}
